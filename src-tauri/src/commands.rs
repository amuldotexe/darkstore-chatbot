use std::sync::Arc;

use tauri::State;

use crate::{
    AppError,
    catalog::EmbeddedCatalogRepository,
    model::OpenAiProductGateway,
    workflow::{
        CartSnapshot, ConciergeWorkflowService, ProductChatContext, RecommendationOutcome,
        SessionStatus,
    },
};

/// Tauri-managed composition root. It owns no secret itself; the workflow owns the session-only
/// API key and clears it on replacement or explicit session reset.
pub struct AppServices {
    workflow_service: ConciergeWorkflowService,
}

pub fn create_runtime_app_services() -> AppServices {
    let catalog_repository =
        Arc::new(EmbeddedCatalogRepository::create_embedded_catalog_repository());
    let product_selection_model = Arc::new(OpenAiProductGateway::create_openai_product_gateway());
    AppServices {
        workflow_service: ConciergeWorkflowService::create_concierge_workflow_service(
            catalog_repository,
            product_selection_model,
        ),
    }
}

impl AppServices {
    pub async fn configure_session_openai_key(
        &self,
        api_key: impl AsRef<str>,
    ) -> Result<SessionStatus, AppError> {
        self.workflow_service
            .configure_session_openai_key(api_key)
            .await
    }

    pub async fn load_initial_product_trio(&self) -> Result<RecommendationOutcome, AppError> {
        self.workflow_service.load_initial_product_trio().await
    }

    pub async fn search_portfolio_products_page(
        &self,
        brief: impl AsRef<str>,
        show_next_page: bool,
    ) -> Result<RecommendationOutcome, AppError> {
        self.workflow_service
            .search_portfolio_products_page(brief, show_next_page)
            .await
    }

    pub async fn select_product_chat_context(
        &self,
        product_sku: impl AsRef<str>,
        selection_source: impl AsRef<str>,
    ) -> Result<ProductChatContext, AppError> {
        self.workflow_service
            .select_product_chat_context(product_sku, selection_source)
            .await
    }

    pub async fn update_product_variant_selection(
        &self,
        product_sku: impl AsRef<str>,
        selected_size: impl AsRef<str>,
    ) -> Result<(), AppError> {
        self.workflow_service
            .update_product_variant_selection(product_sku, selected_size)
            .await
    }

    pub async fn add_validated_variant_cart(
        &self,
        product_sku: impl AsRef<str>,
        selected_size: impl AsRef<str>,
    ) -> Result<CartSnapshot, AppError> {
        self.workflow_service
            .add_validated_variant_cart(product_sku, selected_size)
            .await
    }

    pub async fn clear_session_secret_state(&self) {
        self.workflow_service.clear_session_secret_state().await;
    }
}

#[tauri::command]
pub async fn configure_session_openai_key(
    api_key: String,
    state: State<'_, AppServices>,
) -> Result<SessionStatus, AppError> {
    state.configure_session_openai_key(api_key).await
}

#[tauri::command]
pub async fn load_initial_product_trio(
    state: State<'_, AppServices>,
) -> Result<RecommendationOutcome, AppError> {
    state.load_initial_product_trio().await
}

#[tauri::command]
pub async fn search_portfolio_products_page(
    brief: String,
    show_next_page: bool,
    state: State<'_, AppServices>,
) -> Result<RecommendationOutcome, AppError> {
    state
        .search_portfolio_products_page(brief, show_next_page)
        .await
}

#[tauri::command]
pub async fn select_product_chat_context(
    product_sku: String,
    selection_source: String,
    state: State<'_, AppServices>,
) -> Result<ProductChatContext, AppError> {
    state
        .select_product_chat_context(product_sku, selection_source)
        .await
}

#[tauri::command]
pub async fn update_product_variant_selection(
    product_sku: String,
    selected_size: String,
    state: State<'_, AppServices>,
) -> Result<(), AppError> {
    state
        .update_product_variant_selection(product_sku, selected_size)
        .await
}

#[tauri::command]
pub async fn add_validated_variant_cart(
    product_sku: String,
    selected_size: String,
    state: State<'_, AppServices>,
) -> Result<CartSnapshot, AppError> {
    state
        .add_validated_variant_cart(product_sku, selected_size)
        .await
}

#[tauri::command]
pub async fn clear_session_secret_state(state: State<'_, AppServices>) -> Result<(), AppError> {
    state.clear_session_secret_state().await;
    Ok(())
}
