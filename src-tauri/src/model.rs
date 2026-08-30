use async_trait::async_trait;
use reqwest::Client;
use serde_json::{Value, json};

use crate::{AppError, catalog::CategoryDecision};

#[async_trait]
pub trait CategoryModel: Send + Sync {
    async fn classify_runtime_catalog_category(
        &self,
        api_key: &str,
        taxonomy: &[String],
        brief: &str,
    ) -> Result<CategoryDecision, AppError>;
}

/// Rust-only adapter for the OpenAI Responses API.
///
/// The adapter sends a supplied session key over HTTPS but never retains, logs, or returns it.
/// The WebView receives only the parsed category decision through the command layer.
pub struct OpenAiCategoryGateway {
    http_client: Client,
}

impl OpenAiCategoryGateway {
    pub fn create_openai_category_gateway() -> Self {
        Self {
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl CategoryModel for OpenAiCategoryGateway {
    async fn classify_runtime_catalog_category(
        &self,
        api_key: &str,
        taxonomy: &[String],
        brief: &str,
    ) -> Result<CategoryDecision, AppError> {
        if !api_key.trim().starts_with("sk-") || api_key.trim().len() < 12 {
            return Err(AppError::InvalidApiKey);
        }

        let response = self
            .http_client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(api_key.trim())
            .json(&create_category_request_payload(taxonomy, brief))
            .send()
            .await
            .map_err(|_| AppError::ModelUnavailable)?;

        if !response.status().is_success() {
            return Err(map_openai_failure_status(response.status().as_u16()));
        }

        let response = response
            .json::<Value>()
            .await
            .map_err(|_| AppError::InvalidModelResponse)?;

        parse_openai_category_response(&response)
    }
}

pub fn map_openai_failure_status(status_code: u16) -> AppError {
    match status_code {
        401 => AppError::ModelAuthenticationFailed,
        403 | 404 => AppError::ModelAccessDenied,
        429 => AppError::ModelRateLimited,
        400..=499 => AppError::ModelRequestRejected,
        _ => AppError::ModelUnavailable,
    }
}

pub fn create_category_request_payload(taxonomy: &[String], brief: &str) -> Value {
    let permitted_categories = taxonomy.join(", ");
    json!({
        "model": "gpt-4o",
        "store": false,
        "instructions": format!(
            "You classify a shopper brief against the runtime inventory taxonomy. \
             The only permitted category IDs are: [{permitted_categories}]. \
             Return matched only when the brief calls for exactly one listed category. \
             Otherwise return not_in_inventory. Never return product IDs, prices, inventory, \
             scores, or extra fields."
        ),
        "input": brief,
        "text": {
            "format": {
                "type": "json_schema",
                "name": "inventory_category_decision",
                "strict": true,
                "schema": {
                    "type": "object",
                    "properties": {
                        "kind": {
                            "type": "string",
                            "enum": ["matched", "not_in_inventory"]
                        },
                        "category_id": { "type": ["string", "null"] },
                        "rationale": { "type": ["string", "null"] },
                        "acknowledgement": { "type": ["string", "null"] }
                    },
                    "required": ["kind", "category_id", "rationale", "acknowledgement"],
                    "additionalProperties": false
                }
            }
        }
    })
}

pub fn parse_openai_category_response(response: &Value) -> Result<CategoryDecision, AppError> {
    let output_text = extract_openai_response_text(response)?;
    serde_json::from_str(output_text).map_err(|_| AppError::InvalidModelResponse)
}

fn extract_openai_response_text(response: &Value) -> Result<&str, AppError> {
    extract_top_level_output_text(response)
        .or_else(|| extract_response_output_text(response))
        .ok_or(AppError::InvalidModelResponse)
}

fn extract_top_level_output_text(response: &Value) -> Option<&str> {
    response
        .get("output_text")
        .and_then(Value::as_str)
        .filter(|output_text| !output_text.trim().is_empty())
}

fn extract_response_output_text(response: &Value) -> Option<&str> {
    let output_items = response.get("output")?.as_array()?;
    for output_item in output_items {
        if output_item.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }

        let content_items = match output_item.get("content").and_then(Value::as_array) {
            Some(content_items) => content_items,
            None => continue,
        };
        for content_item in content_items {
            if content_item.get("type").and_then(Value::as_str) != Some("output_text") {
                continue;
            }
            if let Some(output_text) = content_item
                .get("text")
                .and_then(Value::as_str)
                .filter(|output_text| !output_text.trim().is_empty())
            {
                return Some(output_text);
            }
        }
    }
    None
}
