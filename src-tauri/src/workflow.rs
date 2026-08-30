use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    AppError,
    catalog::{
        CatalogProduct, CatalogRepository, CategoryDecision, determine_complete_page_availability,
        rank_category_product_propensity, validate_model_category_decision,
    },
    model::CategoryModel,
};

const FIRST_LOOK_CONTEXT: &str =
    "Seeded shopper preference and trend context: help me choose a dress for tonight.";
const FIRST_LOOK_STREAM: &str = "first_look";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RecommendationOutcome {
    pub kind: String,
    pub category_id: Option<String>,
    pub rationale: String,
    pub cards: Vec<CatalogProduct>,
    pub show_next_three: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProductChatContext {
    pub product: CatalogProduct,
    pub selection_source: String,
    pub retained_brief: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CartLine {
    pub product: CatalogProduct,
    pub selected_size: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CartSnapshot {
    pub item_count: usize,
    pub items: Vec<CartLine>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SessionStatus {
    pub concierge_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectedVariantState {
    product_sku: String,
    selected_size: Option<String>,
    selection_source: String,
}

#[derive(Default)]
struct ConciergeSessionState {
    session_generation: u64,
    api_key: Option<String>,
    retained_brief: String,
    shown_skus_by_stream: HashMap<String, Vec<String>>,
    selected_variant: Option<SelectedVariantState>,
    cart_items: Vec<CartLine>,
}

/// L2 state coordinator over an injected L1 catalog and model port.
///
/// It deliberately copies small session values before awaiting a provider, then checks the
/// generation again before committing user-visible state. This prevents a late answer from a
/// cleared or replaced session from mutating the new session.
pub struct ConciergeWorkflowService {
    catalog_repository: Arc<dyn CatalogRepository>,
    category_model: Arc<dyn CategoryModel>,
    session_state: Mutex<ConciergeSessionState>,
}

impl ConciergeWorkflowService {
    pub fn create_concierge_workflow_service(
        catalog_repository: Arc<dyn CatalogRepository>,
        category_model: Arc<dyn CategoryModel>,
    ) -> Self {
        Self {
            catalog_repository,
            category_model,
            session_state: Mutex::new(ConciergeSessionState::default()),
        }
    }

    pub async fn configure_session_openai_key(
        &self,
        api_key: impl AsRef<str>,
    ) -> Result<SessionStatus, AppError> {
        let trimmed_key = api_key.as_ref().trim();
        if !trimmed_key.starts_with("sk-") || trimmed_key.len() < 12 {
            return Err(AppError::InvalidApiKey);
        }

        let mut state = self.session_state.lock().await;
        state.session_generation = state.session_generation.wrapping_add(1);
        state.api_key = Some(trimmed_key.to_owned());
        state.retained_brief.clear();
        state.shown_skus_by_stream.clear();
        state.selected_variant = None;
        state.cart_items.clear();

        Ok(SessionStatus {
            concierge_enabled: true,
        })
    }

    pub async fn load_initial_product_trio(&self) -> Result<RecommendationOutcome, AppError> {
        self.load_category_product_trio(FIRST_LOOK_CONTEXT, FIRST_LOOK_STREAM, true)
            .await
    }

    pub async fn search_portfolio_products_page(
        &self,
        brief: impl AsRef<str>,
        show_next_page: bool,
    ) -> Result<RecommendationOutcome, AppError> {
        let brief = brief.as_ref().trim();
        if brief.is_empty() {
            return Err(AppError::InvalidCategoryDecision);
        }

        let stream_identifier = format!("brief:{brief}");
        self.load_category_product_trio(brief, &stream_identifier, !show_next_page)
            .await
    }

    pub async fn select_product_chat_context(
        &self,
        product_sku: impl AsRef<str>,
        selection_source: impl AsRef<str>,
    ) -> Result<ProductChatContext, AppError> {
        let product_sku = product_sku.as_ref().trim().to_owned();
        let selection_source = selection_source.as_ref().trim().to_owned();
        if product_sku.is_empty() || selection_source.is_empty() {
            return Err(AppError::ProductUnavailable);
        }

        let session_generation = self.copy_active_session_generation().await?;
        let records = self
            .catalog_repository
            .load_catalog_product_records()
            .await?;
        let product = records
            .into_iter()
            .find(|record| record.sku.as_str() == product_sku && record.fixture_available)
            .ok_or(AppError::ProductUnavailable)?;

        let mut state = self.session_state.lock().await;
        Self::require_matching_session_generation(&state, session_generation)?;
        state.selected_variant = Some(SelectedVariantState {
            product_sku,
            selected_size: None,
            selection_source: selection_source.clone(),
        });

        Ok(ProductChatContext {
            product,
            selection_source,
            retained_brief: state.retained_brief.clone(),
        })
    }

    pub async fn update_product_variant_selection(
        &self,
        product_sku: impl AsRef<str>,
        selected_size: impl AsRef<str>,
    ) -> Result<(), AppError> {
        let product_sku = product_sku.as_ref().trim().to_owned();
        let selected_size = selected_size.as_ref().trim().to_owned();
        let session_generation = self.copy_active_session_generation().await?;
        let records = self
            .catalog_repository
            .load_catalog_product_records()
            .await?;
        let product = records
            .iter()
            .find(|record| record.sku.as_str() == product_sku && record.fixture_available)
            .ok_or(AppError::ProductUnavailable)?;

        if selected_size.is_empty()
            || !product
                .fixture_sizes
                .iter()
                .any(|size| size == &selected_size)
        {
            return Err(AppError::ProductUnavailable);
        }

        let mut state = self.session_state.lock().await;
        Self::require_matching_session_generation(&state, session_generation)?;
        let selected_variant = state
            .selected_variant
            .as_mut()
            .filter(|selection| selection.product_sku == product_sku)
            .ok_or(AppError::ProductUnavailable)?;
        selected_variant.selected_size = Some(selected_size);

        Ok(())
    }

    pub async fn add_validated_variant_cart(
        &self,
        product_sku: impl AsRef<str>,
        selected_size: impl AsRef<str>,
    ) -> Result<CartSnapshot, AppError> {
        let product_sku = product_sku.as_ref().trim().to_owned();
        let selected_size = selected_size.as_ref().trim().to_owned();
        let session_generation = self.copy_active_session_generation().await?;

        {
            let state = self.session_state.lock().await;
            Self::require_matching_session_generation(&state, session_generation)?;
            let selection_matches = state.selected_variant.as_ref().is_some_and(|selection| {
                selection.product_sku == product_sku
                    && selection.selected_size.as_deref() == Some(selected_size.as_str())
            });
            if !selection_matches {
                return Err(AppError::ProductUnavailable);
            }
        }

        let records = self
            .catalog_repository
            .load_catalog_product_records()
            .await?;
        let product = records
            .into_iter()
            .find(|record| record.sku.as_str() == product_sku && record.fixture_available)
            .ok_or(AppError::ProductUnavailable)?;
        if !product
            .fixture_sizes
            .iter()
            .any(|size| size == &selected_size)
        {
            return Err(AppError::ProductUnavailable);
        }

        let mut state = self.session_state.lock().await;
        Self::require_matching_session_generation(&state, session_generation)?;
        let selection_matches = state.selected_variant.as_ref().is_some_and(|selection| {
            selection.product_sku == product_sku
                && selection.selected_size.as_deref() == Some(selected_size.as_str())
        });
        if !selection_matches {
            return Err(AppError::ProductUnavailable);
        }

        if !state.cart_items.iter().any(|line| {
            line.product.sku.as_str() == product_sku && line.selected_size == selected_size
        }) {
            state.cart_items.push(CartLine {
                product,
                selected_size,
            });
        }

        Ok(CartSnapshot {
            item_count: state.cart_items.len(),
            items: state.cart_items.clone(),
        })
    }

    pub async fn clear_session_secret_state(&self) {
        let mut state = self.session_state.lock().await;
        state.session_generation = state.session_generation.wrapping_add(1);
        state.api_key = None;
        state.retained_brief.clear();
        state.shown_skus_by_stream.clear();
        state.selected_variant = None;
        state.cart_items.clear();
    }

    async fn load_category_product_trio(
        &self,
        brief: &str,
        stream_identifier: &str,
        reset_stream: bool,
    ) -> Result<RecommendationOutcome, AppError> {
        let (api_key, session_generation, shown_skus) = {
            let state = self.session_state.lock().await;
            let api_key = state.api_key.clone().ok_or(AppError::SessionUnavailable)?;
            let shown_skus = if reset_stream {
                Vec::new()
            } else {
                state
                    .shown_skus_by_stream
                    .get(stream_identifier)
                    .cloned()
                    .unwrap_or_default()
            };
            (api_key, state.session_generation, shown_skus)
        };

        let taxonomy = self
            .catalog_repository
            .load_runtime_inventory_taxonomy()
            .await?;
        let model_decision = self
            .category_model
            .classify_runtime_catalog_category(&api_key, &taxonomy, brief)
            .await?;
        let validated_decision = validate_model_category_decision(&taxonomy, model_decision)?;

        let (category_id, rationale) = match validated_decision {
            CategoryDecision::Matched {
                category_id,
                rationale,
            } => (category_id, rationale),
            CategoryDecision::NotInInventory { acknowledgement } => {
                let mut state = self.session_state.lock().await;
                Self::require_matching_session_generation(&state, session_generation)?;
                state.retained_brief = brief.to_owned();
                return Ok(RecommendationOutcome {
                    kind: "not_in_inventory".to_owned(),
                    category_id: None,
                    rationale: acknowledgement,
                    cards: Vec::new(),
                    show_next_three: false,
                });
            }
        };

        let records = self
            .catalog_repository
            .load_catalog_product_records()
            .await?;
        let cards = rank_category_product_propensity(&records, &category_id, &shown_skus)?;
        let mut visible_skus = shown_skus;
        visible_skus.extend(cards.iter().map(|card| card.sku.as_str().to_owned()));
        let show_next_three =
            determine_complete_page_availability(&records, &category_id, &visible_skus);

        let mut state = self.session_state.lock().await;
        Self::require_matching_session_generation(&state, session_generation)?;
        state.retained_brief = brief.to_owned();
        state
            .shown_skus_by_stream
            .insert(stream_identifier.to_owned(), visible_skus);

        Ok(RecommendationOutcome {
            kind: "cards".to_owned(),
            category_id: Some(category_id),
            rationale,
            cards,
            show_next_three,
        })
    }

    async fn copy_active_session_generation(&self) -> Result<u64, AppError> {
        let state = self.session_state.lock().await;
        if state.api_key.is_none() {
            return Err(AppError::SessionUnavailable);
        }
        Ok(state.session_generation)
    }

    fn require_matching_session_generation(
        state: &ConciergeSessionState,
        expected_generation: u64,
    ) -> Result<(), AppError> {
        if state.api_key.is_none() || state.session_generation != expected_generation {
            return Err(AppError::SessionUnavailable);
        }
        Ok(())
    }
}
