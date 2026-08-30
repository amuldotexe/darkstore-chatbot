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
            .map_err(|_| AppError::ModelUnavailable)?
            .error_for_status()
            .map_err(|_| AppError::ModelUnavailable)?
            .json::<Value>()
            .await
            .map_err(|_| AppError::InvalidModelResponse)?;

        parse_openai_category_response(&response)
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
                    "oneOf": [
                        {
                            "type": "object",
                            "properties": {
                                "kind": { "const": "matched" },
                                "category_id": { "type": "string" },
                                "rationale": { "type": "string" }
                            },
                            "required": ["kind", "category_id", "rationale"],
                            "additionalProperties": false
                        },
                        {
                            "type": "object",
                            "properties": {
                                "kind": { "const": "not_in_inventory" },
                                "acknowledgement": { "type": "string" }
                            },
                            "required": ["kind", "acknowledgement"],
                            "additionalProperties": false
                        }
                    ]
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
    response
        .get("output_text")
        .and_then(Value::as_str)
        .filter(|output_text| !output_text.trim().is_empty())
        .ok_or(AppError::InvalidModelResponse)
}
