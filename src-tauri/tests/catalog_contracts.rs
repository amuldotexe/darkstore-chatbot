use darkstore_concierge::catalog::{
    CatalogProduct, CategoryDecision, rank_category_product_propensity,
    validate_model_category_decision,
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
fn test_req_tauri_003_accepts_only_dress_category_decision() {
    let taxonomy = ["dresses".to_owned()];

    let decision = validate_model_category_decision(
        &taxonomy,
        CategoryDecision::Matched {
            category_id: "dresses".to_owned(),
            rationale: "A party dress fits the brief.".to_owned(),
        },
    )
    .expect("valid dress decision should pass");

    assert!(matches!(decision, CategoryDecision::Matched { .. }));
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
