use darkstore_concierge::catalog::{
    CatalogProduct, CatalogRepository, EmbeddedCatalogRepository, rank_category_product_propensity,
    rank_remaining_product_page, resolve_model_selected_products,
};

fn create_catalog_fixture_records() -> Vec<CatalogProduct> {
    vec![
        CatalogProduct::create_fixture_catalog_product("SKID00083927", "dresses", 94)
            .expect("fixture SKU is valid"),
        CatalogProduct::create_fixture_catalog_product("SKID00174036", "dresses", 92)
            .expect("fixture SKU is valid"),
        CatalogProduct::create_fixture_catalog_product("SKID00081801", "dresses", 89)
            .expect("fixture SKU is valid"),
        CatalogProduct::create_fixture_catalog_product("SKID00207435", "dresses", 87)
            .expect("fixture SKU is valid"),
        CatalogProduct::create_fixture_catalog_product("SKID00119053", "dresses", 85)
            .expect("fixture SKU is valid"),
        CatalogProduct::create_fixture_catalog_product("SKID00184392", "dresses", 82)
            .expect("fixture SKU is valid"),
        CatalogProduct::create_fixture_catalog_product("SKID00076560", "dresses", 80)
            .expect("fixture SKU is valid"),
        CatalogProduct::create_fixture_catalog_product("SKID00167395", "dresses", 80)
            .expect("fixture SKU is valid"),
    ]
}

#[test]
fn test_req_tauri_013_ranks_three_products_by_score_then_sku() {
    let records = create_catalog_fixture_records();

    let trio = rank_category_product_propensity(&records, "dresses", &[])
        .expect("the fixture contains a complete trio");

    let identifiers: Vec<_> = trio.iter().map(|product| product.sku.as_str()).collect();
    assert_eq!(
        identifiers,
        ["SKID00083927", "SKID00174036", "SKID00081801"]
    );
}

#[test]
fn test_req_tauri_027_rejects_duplicate_or_unknown_model_skus() {
    let candidates = create_catalog_fixture_records();
    let duplicate = resolve_model_selected_products(
        &candidates,
        &[
            "SKID00083927".to_owned(),
            "SKID00083927".to_owned(),
            "SKID00081801".to_owned(),
        ],
    );
    let unknown = resolve_model_selected_products(
        &candidates,
        &[
            "SKID00083927".to_owned(),
            "SKID00174036".to_owned(),
            "NOT-OFFERED".to_owned(),
        ],
    );

    assert_eq!(
        duplicate.expect_err("duplicate selection must fail").kind(),
        "invalid_model_response"
    );
    assert_eq!(
        unknown.expect_err("unknown selection must fail").kind(),
        "invalid_model_response"
    );
}

#[test]
fn test_req_tauri_016_rejects_an_incomplete_next_page() {
    let records = create_catalog_fixture_records();
    let shown = [
        "SKID00083927".to_owned(),
        "SKID00174036".to_owned(),
        "SKID00081801".to_owned(),
        "SKID00207435".to_owned(),
        "SKID00119053".to_owned(),
        "SKID00184392".to_owned(),
    ];

    let error = rank_category_product_propensity(&records, "dresses", &shown)
        .expect_err("two remaining products must not make a partial page");

    assert_eq!(error.kind(), "complete_page_exhausted");
}

#[test]
fn test_req_tauri_024_returns_final_partial_inventory_page() {
    let records = create_catalog_fixture_records();
    let shown = [
        "SKID00083927".to_owned(),
        "SKID00174036".to_owned(),
        "SKID00081801".to_owned(),
        "SKID00207435".to_owned(),
        "SKID00119053".to_owned(),
        "SKID00184392".to_owned(),
    ];

    let final_page = rank_remaining_product_page(&records, "dresses", &shown)
        .expect("the last two dresses should be returned");

    assert_eq!(final_page.len(), 2);
    assert_eq!(final_page[0].sku.as_str(), "SKID00076560");
    assert_eq!(final_page[1].sku.as_str(), "SKID00167395");
}

#[tokio::test]
async fn test_req_tauri_017_embedded_catalogue_loads_without_turso_configuration() {
    let repository = EmbeddedCatalogRepository::create_embedded_catalog_repository();

    let taxonomy = repository
        .load_runtime_inventory_taxonomy()
        .await
        .expect("the embedded catalogue must not need launch-shell configuration");
    let products = repository
        .load_catalog_product_records()
        .await
        .expect("the embedded catalogue must be available before any network work");

    assert_eq!(taxonomy, vec!["dresses"]);
    assert_eq!(products.len(), 8);
    assert_eq!(
        products[0].product_name,
        "Black Minimalist A-Line Evening Dress For Date Night"
    );
}
