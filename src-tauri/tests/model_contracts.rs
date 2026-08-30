use darkstore_concierge::{
    AppError, catalog::CategoryDecision, model::parse_openai_category_response,
};
use serde_json::json;

#[test]
fn test_req_tauri_003_parses_structured_category_output_only() {
    let response = json!({
        "output_text": "{\"kind\":\"matched\",\"category_id\":\"dresses\",\"rationale\":\"The brief calls for a dress.\"}"
    });

    let decision = parse_openai_category_response(&response)
        .expect("a structured category result should parse without product facts");

    assert_eq!(
        decision,
        CategoryDecision::Matched {
            category_id: "dresses".to_owned(),
            rationale: "The brief calls for a dress.".to_owned(),
        }
    );
}

#[test]
fn test_req_tauri_010_rejects_missing_or_non_json_model_output() {
    let missing_output = parse_openai_category_response(&json!({"id": "resp_fixture"}));
    let invalid_output = parse_openai_category_response(&json!({"output_text": "not json"}));

    assert_eq!(missing_output, Err(AppError::InvalidModelResponse));
    assert_eq!(invalid_output, Err(AppError::InvalidModelResponse));
}
