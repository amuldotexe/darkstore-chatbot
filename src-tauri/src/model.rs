use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{AppError, catalog::CatalogProduct};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductSelection {
    pub selected_skus: Vec<String>,
    pub rationale: String,
}

#[async_trait]
pub trait ProductSelectionModel: Send + Sync {
    async fn select_available_product_skus(
        &self,
        api_key: &str,
        candidates: &[CatalogProduct],
        brief: &str,
    ) -> Result<ProductSelection, AppError>;
}

/// Rust-only adapter for GPT-4o product selection from the supplied inventory snapshot.
///
/// The shopper key stays in session-only workflow state. This adapter uses it only for the
/// outbound HTTPS request and never logs, serializes, or retains it.
pub struct OpenAiProductGateway {
    http_client: Client,
}

impl OpenAiProductGateway {
    pub fn create_openai_product_gateway() -> Self {
        Self {
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl ProductSelectionModel for OpenAiProductGateway {
    async fn select_available_product_skus(
        &self,
        api_key: &str,
        candidates: &[CatalogProduct],
        brief: &str,
    ) -> Result<ProductSelection, AppError> {
        if !api_key.trim().starts_with("sk-") || api_key.trim().len() < 12 {
            return Err(AppError::InvalidApiKey);
        }

        let response = self
            .http_client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(api_key.trim())
            .json(&create_product_selection_request_payload(candidates, brief))
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

        parse_openai_product_selection(&response)
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

pub fn create_product_selection_request_payload(
    candidates: &[CatalogProduct],
    brief: &str,
) -> Value {
    let required_count = candidates.len().min(3);
    let available_dresses = candidates
        .iter()
        .map(|product| {
            json!({
                "sku": product.sku.as_str(),
                "brand": product.brand,
                "product_name": product.product_name,
                "sizes": product.fixture_sizes,
                "dress_type": product.fixture_dress_type,
                "style_tags": product.fixture_style_tags,
                "price_inr": product.current_price_inr,
                "delivery_minutes": product.fixture_delivery_minutes,
                "propensity_score": product.fixture_propensity_score
            })
        })
        .collect::<Vec<_>>();
    json!({
        "model": "gpt-4o",
        "store": false,
        "instructions": format!(
            "You are a fashion concierge for a dress-only demo store. Interpret the shopper's \
             words as preferences, including colour, mood, event, silhouette, or style. Select \
             exactly {required_count} different SKU values from available_dresses, in best-first \
             order. Never invent a SKU, return a product category, claim no inventory, or add \
             fields beyond the strict response schema."
        ),
        "input": {
            "shopper_brief": brief,
            "available_dresses": available_dresses
        },
        "text": {
            "format": {
                "type": "json_schema",
                "name": "available_dress_selection",
                "strict": true,
                "schema": {
                    "type": "object",
                    "properties": {
                        "selected_skus": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "rationale": { "type": "string" }
                    },
                    "required": ["selected_skus", "rationale"],
                    "additionalProperties": false
                }
            }
        }
    })
}

pub fn parse_openai_product_selection(response: &Value) -> Result<ProductSelection, AppError> {
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
