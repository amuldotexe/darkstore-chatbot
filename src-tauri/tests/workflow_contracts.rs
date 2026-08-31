use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use darkstore_concierge::{
    AppError,
    catalog::{CatalogProduct, CatalogRepository, EmbeddedCatalogRepository},
    model::{ProductSelection, ProductSelectionModel},
    workflow::ConciergeWorkflowService,
};

#[derive(Clone)]
struct FixtureCatalogRepository {
    records: Vec<CatalogProduct>,
    unavailable: bool,
}

#[async_trait]
impl CatalogRepository for FixtureCatalogRepository {
    async fn load_runtime_inventory_taxonomy(&self) -> Result<Vec<String>, AppError> {
        if self.unavailable {
            return Err(AppError::InventoryUnavailable);
        }
        Ok(vec!["dresses".to_owned()])
    }

    async fn load_catalog_product_records(&self) -> Result<Vec<CatalogProduct>, AppError> {
        if self.unavailable {
            return Err(AppError::InventoryUnavailable);
        }
        Ok(self.records.clone())
    }
}

#[derive(Clone, Copy)]
enum FixtureSelectionBehavior {
    Valid,
    Invalid,
    Unavailable,
}

struct FixtureProductSelectionModel {
    calls: Arc<AtomicUsize>,
    behavior: FixtureSelectionBehavior,
}

#[async_trait]
impl ProductSelectionModel for FixtureProductSelectionModel {
    async fn select_available_product_skus(
        &self,
        _api_key: &str,
        candidates: &[CatalogProduct],
        _brief: &str,
    ) -> Result<ProductSelection, AppError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        match self.behavior {
            FixtureSelectionBehavior::Valid => Ok(ProductSelection {
                selected_skus: candidates
                    .iter()
                    .take(3)
                    .map(|candidate| candidate.sku.as_str().to_owned())
                    .collect(),
                rationale: "The available dresses best match the brief.".to_owned(),
            }),
            FixtureSelectionBehavior::Invalid => Ok(ProductSelection {
                selected_skus: vec!["UNLISTED-SKU".to_owned()],
                rationale: "This must be discarded by the Rust validator.".to_owned(),
            }),
            FixtureSelectionBehavior::Unavailable => Err(AppError::ModelUnavailable),
        }
    }
}

struct RecheckingCatalogRepository {
    records: Vec<CatalogProduct>,
    catalog_reads: AtomicUsize,
}

#[async_trait]
impl CatalogRepository for RecheckingCatalogRepository {
    async fn load_runtime_inventory_taxonomy(&self) -> Result<Vec<String>, AppError> {
        Ok(vec!["dresses".to_owned()])
    }

    async fn load_catalog_product_records(&self) -> Result<Vec<CatalogProduct>, AppError> {
        let read_number = self.catalog_reads.fetch_add(1, Ordering::SeqCst);
        let mut records = self.records.clone();
        if read_number >= 2
            && let Some(product) = records
                .iter_mut()
                .find(|product| product.sku.as_str() == "SKID00083927")
        {
            product.fixture_available = false;
        }
        Ok(records)
    }
}

fn create_workflow_fixture_records() -> Vec<CatalogProduct> {
    [
        ("SKID00083927", 94),
        ("SKID00174036", 92),
        ("SKID00081801", 89),
        ("SKID00207435", 87),
        ("SKID00119053", 85),
        ("SKID00184392", 82),
        ("SKID00076560", 80),
        ("SKID00167395", 80),
    ]
    .into_iter()
    .map(|(sku, score)| {
        CatalogProduct::create_fixture_catalog_product(sku, "dresses", score)
            .expect("fixture SKU is valid")
    })
    .collect()
}

fn create_fixture_product_selector(
    calls: Arc<AtomicUsize>,
    behavior: FixtureSelectionBehavior,
) -> Arc<FixtureProductSelectionModel> {
    Arc::new(FixtureProductSelectionModel { calls, behavior })
}

#[tokio::test]
async fn test_req_tauri_026_to_027_returns_validated_model_ordered_cards() {
    let calls = Arc::new(AtomicUsize::new(0));
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(EmbeddedCatalogRepository::create_embedded_catalog_repository()),
        create_fixture_product_selector(Arc::clone(&calls), FixtureSelectionBehavior::Valid),
    );

    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("syntactically valid key unlocks this in-memory session");
    let first_look = workflow
        .load_initial_product_trio()
        .await
        .expect("a valid model result should yield cards");

    assert_eq!(first_look.kind, "cards");
    assert_eq!(first_look.cards.len(), 3);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        first_look.cards[0].sku.as_str(),
        "SKID00083927",
        "the validated model order is preserved"
    );
}

#[tokio::test]
async fn test_req_tauri_028_falls_back_when_model_result_is_unusable() {
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(FixtureCatalogRepository {
            records: create_workflow_fixture_records(),
            unavailable: false,
        }),
        create_fixture_product_selector(
            Arc::new(AtomicUsize::new(0)),
            FixtureSelectionBehavior::Invalid,
        ),
    );

    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("key is valid");
    let outcome = workflow
        .search_portfolio_products_page("black", false)
        .await
        .expect("invalid model SKU output must use local fallback cards");

    assert_eq!(outcome.kind, "cards");
    assert_eq!(outcome.cards.len(), 3);
    assert_eq!(outcome.cards[0].sku.as_str(), "SKID00083927");
    assert!(
        outcome
            .rationale
            .contains("ranked from the current inventory")
    );
}

#[tokio::test]
async fn test_req_tauri_029_returns_dresses_for_style_only_brief_when_model_is_down() {
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(FixtureCatalogRepository {
            records: create_workflow_fixture_records(),
            unavailable: false,
        }),
        create_fixture_product_selector(
            Arc::new(AtomicUsize::new(0)),
            FixtureSelectionBehavior::Unavailable,
        ),
    );

    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("key is valid");
    let outcome = workflow
        .search_portfolio_products_page("red", false)
        .await
        .expect("a style-only brief still receives three dresses");

    assert_eq!(outcome.cards.len(), 3);
    assert!(
        outcome
            .cards
            .iter()
            .all(|card| card.category_id == "dresses")
    );
}

#[tokio::test]
async fn test_req_tauri_030_returns_final_partial_page_then_exhausts_inventory() {
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(FixtureCatalogRepository {
            records: create_workflow_fixture_records(),
            unavailable: false,
        }),
        create_fixture_product_selector(
            Arc::new(AtomicUsize::new(0)),
            FixtureSelectionBehavior::Valid,
        ),
    );
    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("key is valid");

    let first_page = workflow
        .search_portfolio_products_page("black", false)
        .await
        .expect("first page exists");
    let second_page = workflow
        .search_portfolio_products_page("black", true)
        .await
        .expect("second page exists");
    let final_page = workflow
        .search_portfolio_products_page("black", true)
        .await
        .expect("final partial page exists");
    let exhausted_page = workflow
        .search_portfolio_products_page("black", true)
        .await
        .expect_err("fourth request must report exhausted inventory");

    assert_eq!(first_page.cards.len(), 3);
    assert_eq!(second_page.cards.len(), 3);
    assert_eq!(final_page.cards.len(), 2);
    assert!(!final_page.show_next_three);
    assert_eq!(exhausted_page.kind(), "complete_page_exhausted");
}

#[tokio::test]
async fn test_req_tauri_015_stops_before_model_when_inventory_is_unavailable() {
    let calls = Arc::new(AtomicUsize::new(0));
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(FixtureCatalogRepository {
            records: Vec::new(),
            unavailable: true,
        }),
        create_fixture_product_selector(Arc::clone(&calls), FixtureSelectionBehavior::Valid),
    );

    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("key is valid");
    let error = workflow
        .load_initial_product_trio()
        .await
        .expect_err("unavailable inventory must stop the flow");

    assert_eq!(error.kind(), "inventory_unavailable");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_req_tauri_012_clear_session_rejects_late_cart_mutation() {
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(FixtureCatalogRepository {
            records: create_workflow_fixture_records(),
            unavailable: false,
        }),
        create_fixture_product_selector(
            Arc::new(AtomicUsize::new(0)),
            FixtureSelectionBehavior::Valid,
        ),
    );

    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("key is valid");
    workflow
        .select_product_chat_context("SKID00083927", "first_look")
        .await
        .expect("product can enter chat");
    workflow
        .update_product_variant_selection("SKID00083927", "S")
        .await
        .expect("size is valid");

    workflow.clear_session_secret_state().await;
    let error = workflow
        .add_validated_variant_cart("SKID00083927", "S")
        .await
        .expect_err("cleared state must not accept a stale cart request");

    assert_eq!(error.kind(), "session_unavailable");
}

#[tokio::test]
async fn test_req_tauri_031_rechecks_fixture_availability_before_cart_mutation() {
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(RecheckingCatalogRepository {
            records: create_workflow_fixture_records(),
            catalog_reads: AtomicUsize::new(0),
        }),
        create_fixture_product_selector(
            Arc::new(AtomicUsize::new(0)),
            FixtureSelectionBehavior::Valid,
        ),
    );

    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("key is valid");
    workflow
        .select_product_chat_context("SKID00083927", "first_look")
        .await
        .expect("product can enter chat");
    workflow
        .update_product_variant_selection("SKID00083927", "S")
        .await
        .expect("initial fixture is available");

    let error = workflow
        .add_validated_variant_cart("SKID00083927", "S")
        .await
        .expect_err("cart operation must re-read fixture availability");

    assert_eq!(error.kind(), "product_unavailable");
}
