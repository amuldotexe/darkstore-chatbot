use darkstore_concierge::{
    AppError,
    catalog::CatalogProduct,
    model::{
        create_product_selection_request_payload, map_openai_failure_status,
        parse_openai_product_selection,
    },
};
use serde_json::json;

fn create_model_candidate_records() -> Vec<CatalogProduct> {
    vec![
        CatalogProduct::create_fixture_catalog_product("DRESS-001", "dresses", 94)
            .expect("fixture candidate is valid"),
        CatalogProduct::create_fixture_catalog_product("DRESS-002", "dresses", 92)
            .expect("fixture candidate is valid"),
        CatalogProduct::create_fixture_catalog_product("DRESS-003", "dresses", 89)
            .expect("fixture candidate is valid"),
    ]
}

#[test]
fn test_req_tauri_026_selects_only_offered_dress_skus() {
    let request = create_product_selection_request_payload(
        &create_model_candidate_records(),
        "black wedding dress",
    );

    assert_eq!(request["model"], "gpt-4o");
    assert_eq!(request["store"], false);
    assert_eq!(request["input"]["shopper_brief"], "black wedding dress");
    assert_eq!(request["input"]["available_dresses"][0]["sku"], "DRESS-001");
    assert_eq!(
        request["text"]["format"]["schema"]["required"],
        json!(["selected_skus", "rationale"])
    );
    assert_eq!(
        request["text"]["format"]["schema"]["additionalProperties"],
        false
    );
}

#[test]
fn test_req_tauri_026_parses_strict_product_selection_output() {
    let selection = parse_openai_product_selection(&json!({
        "output_text": "{\"selected_skus\":[\"DRESS-003\",\"DRESS-001\",\"DRESS-002\"],\"rationale\":\"The darker options suit the brief.\"}"
    }))
    .expect("strict product selection parses");

    assert_eq!(
        selection.selected_skus,
        vec!["DRESS-003", "DRESS-001", "DRESS-002"]
    );
    assert_eq!(selection.rationale, "The darker options suit the brief.");
}

#[test]
fn test_req_tauri_028_rejects_missing_or_non_json_model_output() {
    let missing_output = parse_openai_product_selection(&json!({"id": "resp_fixture"}));
    let invalid_output = parse_openai_product_selection(&json!({"output_text": "not json"}));

    assert_eq!(missing_output, Err(AppError::InvalidModelResponse));
    assert_eq!(invalid_output, Err(AppError::InvalidModelResponse));
}

#[test]
fn test_req_tauri_021_parses_raw_responses_api_output_text() {
    let response = json!({
        "id": "resp_fixture",
        "status": "completed",
        "output": [{
            "type": "message",
            "content": [{
                "type": "output_text",
                "text": "{\"selected_skus\":[\"DRESS-001\",\"DRESS-002\",\"DRESS-003\"],\"rationale\":\"A complete selection.\"}"
            }]
        }]
    });

    assert_eq!(
        parse_openai_product_selection(&response)
            .expect("raw Responses API output parses")
            .selected_skus,
        vec!["DRESS-001", "DRESS-002", "DRESS-003"]
    );
}

#[test]
fn test_req_tauri_019_maps_openai_failure_statuses_safely() {
    assert_eq!(
        map_openai_failure_status(401),
        AppError::ModelAuthenticationFailed
    );
    assert_eq!(map_openai_failure_status(403), AppError::ModelAccessDenied);
    assert_eq!(map_openai_failure_status(404), AppError::ModelAccessDenied);
    assert_eq!(map_openai_failure_status(429), AppError::ModelRateLimited);
    assert_eq!(
        map_openai_failure_status(400),
        AppError::ModelRequestRejected
    );
    assert_eq!(map_openai_failure_status(503), AppError::ModelUnavailable);
}
