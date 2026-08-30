use serde::{Serialize, Serializer, ser::SerializeStruct};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppError {
    #[error("Enter a valid OpenAI API key to continue.")]
    InvalidApiKey,
    #[error("The catalogue cannot be reached right now. Please try again.")]
    InventoryUnavailable,
    #[error("The catalogue fixture does not contain a complete three-card set.")]
    InventoryConfiguration,
    #[error("That category is not available in this demo.")]
    InvalidCategoryDecision,
    #[error("There are not three unseen products left for another page.")]
    CompletePageExhausted,
    #[error("That product or size is no longer available in the demo inventory.")]
    ProductUnavailable,
    #[error("The model response did not match the category contract.")]
    InvalidModelResponse,
    #[error("The model request could not be completed. Please try again.")]
    ModelUnavailable,
    #[error("The session state is temporarily unavailable. Please try again.")]
    SessionUnavailable,
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::InvalidApiKey => "invalid_api_key",
            Self::InventoryUnavailable => "inventory_unavailable",
            Self::InventoryConfiguration => "inventory_configuration",
            Self::InvalidCategoryDecision => "invalid_category_decision",
            Self::CompletePageExhausted => "complete_page_exhausted",
            Self::ProductUnavailable => "product_unavailable",
            Self::InvalidModelResponse => "invalid_model_response",
            Self::ModelUnavailable => "model_unavailable",
            Self::SessionUnavailable => "session_unavailable",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serialized_error = serializer.serialize_struct("AppError", 2)?;
        serialized_error.serialize_field("kind", self.kind())?;
        serialized_error.serialize_field("message", &self.to_string())?;
        serialized_error.end()
    }
}
