use darkstore_concierge::AppError;

#[test]
fn test_req_tauri_010_serializes_stable_kind_and_shopper_safe_message() {
    let serialized = serde_json::to_value(AppError::InventoryUnavailable)
        .expect("AppError must cross the command boundary as JSON");

    assert_eq!(serialized["kind"], "inventory_unavailable");
    assert_eq!(
        serialized["message"],
        "The catalogue cannot be reached right now. Please try again."
    );
    assert!(!serialized.to_string().contains("TURSO_AUTH_TOKEN"));
}
