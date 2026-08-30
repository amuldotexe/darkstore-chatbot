use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use darkstore_concierge::{
    AppError,
    catalog::{CatalogProduct, CatalogRepository, CategoryDecision, EmbeddedCatalogRepository},
    model::CategoryModel,
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

struct FixtureCategoryModel {
    calls: Arc<AtomicUsize>,
    decision: CategoryDecision,
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

#[async_trait]
impl CategoryModel for FixtureCategoryModel {
    async fn classify_runtime_catalog_category(
        &self,
        _api_key: &str,
        _taxonomy: &[String],
        _brief: &str,
    ) -> Result<CategoryDecision, AppError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.decision.clone())
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

#[tokio::test]
async fn test_req_tauri_001_to_008_completes_guided_cart_journey() {
    let calls = Arc::new(AtomicUsize::new(0));
    let model = FixtureCategoryModel {
        calls: Arc::clone(&calls),
        decision: CategoryDecision::Matched {
            category_id: "dresses".to_owned(),
            rationale: "A date-night dress is in the current portfolio.".to_owned(),
        },
    };
    let catalog = EmbeddedCatalogRepository::create_embedded_catalog_repository();
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(catalog),
        Arc::new(model),
    );

    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("syntactically valid key unlocks this in-memory session");
    let first_look = workflow
        .load_initial_product_trio()
        .await
        .expect("fixture category yields three cards");

    assert_eq!(first_look.cards.len(), 3);
    assert!(first_look.show_next_three);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let selected = workflow
        .select_product_chat_context("SKID00083927", "first_look")
        .await
        .expect("first card can anchor chat");
    let size = selected
        .product
        .fixture_sizes
        .first()
        .expect("fixture has a selectable size");

    workflow
        .update_product_variant_selection("SKID00083927", size)
        .await
        .expect("available size is accepted");
    let cart = workflow
        .add_validated_variant_cart("SKID00083927", size)
        .await
        .expect("revalidated variant enters the local cart");

    assert_eq!(cart.item_count, 1);
}

#[tokio::test]
async fn test_req_tauri_014_returns_no_cards_for_absent_category() {
    let calls = Arc::new(AtomicUsize::new(0));
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(FixtureCatalogRepository {
            records: create_workflow_fixture_records(),
            unavailable: false,
        }),
        Arc::new(FixtureCategoryModel {
            calls,
            decision: CategoryDecision::NotInInventory {
                acknowledgement: "This v001 demo currently carries dresses only.".to_owned(),
            },
        }),
    );

    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("key is valid");
    let outcome = workflow
        .search_portfolio_products_page("I need a linen shirt", false)
        .await
        .expect("absence is a valid model outcome");

    assert!(outcome.cards.is_empty());
    assert_eq!(outcome.kind, "not_in_inventory");
}

#[tokio::test]
async fn test_req_tauri_015_stops_before_model_when_taxonomy_is_unavailable() {
    let calls = Arc::new(AtomicUsize::new(0));
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(FixtureCatalogRepository {
            records: Vec::new(),
            unavailable: true,
        }),
        Arc::new(FixtureCategoryModel {
            calls: Arc::clone(&calls),
            decision: CategoryDecision::Matched {
                category_id: "dresses".to_owned(),
                rationale: "Never used.".to_owned(),
            },
        }),
    );

    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("key is valid");
    let error = workflow
        .load_initial_product_trio()
        .await
        .expect_err("unavailable taxonomy must stop the flow");

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
        Arc::new(FixtureCategoryModel {
            calls: Arc::new(AtomicUsize::new(0)),
            decision: CategoryDecision::Matched {
                category_id: "dresses".to_owned(),
                rationale: "A dress is available.".to_owned(),
            },
        }),
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
async fn test_req_tauri_007_rechecks_fixture_availability_before_cart_mutation() {
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(RecheckingCatalogRepository {
            records: create_workflow_fixture_records(),
            catalog_reads: AtomicUsize::new(0),
        }),
        Arc::new(FixtureCategoryModel {
            calls: Arc::new(AtomicUsize::new(0)),
            decision: CategoryDecision::Matched {
                category_id: "dresses".to_owned(),
                rationale: "A dress is available.".to_owned(),
            },
        }),
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

#[tokio::test]
async fn test_req_tauri_004_and_016_returns_final_partial_page_then_exhausts_inventory() {
    let workflow = ConciergeWorkflowService::create_concierge_workflow_service(
        Arc::new(FixtureCatalogRepository {
            records: create_workflow_fixture_records(),
            unavailable: false,
        }),
        Arc::new(FixtureCategoryModel {
            calls: Arc::new(AtomicUsize::new(0)),
            decision: CategoryDecision::Matched {
                category_id: "dresses".to_owned(),
                rationale: "A dress is available.".to_owned(),
            },
        }),
    );
    workflow
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("key is valid");

    let first_page = workflow
        .search_portfolio_products_page("A black dress", false)
        .await
        .expect("first complete page exists");
    let second_page = workflow
        .search_portfolio_products_page("A black dress", true)
        .await
        .expect("second complete page exists");
    let final_page = workflow
        .search_portfolio_products_page("A black dress", true)
        .await
        .expect("the final two dresses should render as a partial page");
    let exhausted_page = workflow
        .search_portfolio_products_page("A black dress", true)
        .await
        .expect_err("a fourth request must report exhausted inventory");

    assert_eq!(first_page.cards.len(), 3);
    assert!(first_page.show_next_three);
    assert_eq!(second_page.cards.len(), 3);
    assert!(second_page.show_next_three);
    assert_eq!(final_page.cards.len(), 2);
    assert!(!final_page.show_next_three);
    assert!(first_page.cards.iter().all(|first| {
        second_page
            .cards
            .iter()
            .all(|second| first.sku != second.sku)
    }));
    assert!(second_page.cards.iter().all(|second| {
        final_page
            .cards
            .iter()
            .all(|final_card| second.sku != final_card.sku)
    }));
    assert_eq!(exhausted_page.kind(), "complete_page_exhausted");
}
