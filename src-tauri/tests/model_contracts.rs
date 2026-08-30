use darkstore_concierge::{
    AppError,
    catalog::CategoryDecision,
    model::{
        create_category_request_payload, map_openai_failure_status, parse_openai_category_response,
    },
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

#[test]
fn test_req_tauri_020_uses_strict_root_object_schema_only() {
    let request = create_category_request_payload(&["dresses".to_owned()], "A dress for dinner");
    let schema = &request["text"]["format"]["schema"];

    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    assert!(schema.get("oneOf").is_none());
    assert_eq!(
        schema["required"],
        json!(["kind", "category_id", "rationale", "acknowledgement"])
    );
    assert_eq!(
        schema["properties"]["kind"]["enum"],
        json!(["matched", "not_in_inventory"])
    );
}

#[test]
fn test_req_tauri_020_parses_the_full_strict_schema_match() {
    let response = json!({
        "output_text": "{\"kind\":\"matched\",\"category_id\":\"dresses\",\"rationale\":\"A dress fits the brief.\",\"acknowledgement\":null}"
    });

    assert_eq!(
        parse_openai_category_response(&response),
        Ok(CategoryDecision::Matched {
            category_id: "dresses".to_owned(),
            rationale: "A dress fits the brief.".to_owned(),
        })
    );
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
                "text": "{\"kind\":\"matched\",\"category_id\":\"dresses\",\"rationale\":\"A dress fits the brief.\",\"acknowledgement\":null}"
            }]
        }]
    });

    assert_eq!(
        parse_openai_category_response(&response),
        Ok(CategoryDecision::Matched {
            category_id: "dresses".to_owned(),
            rationale: "A dress fits the brief.".to_owned(),
        })
    );
}
