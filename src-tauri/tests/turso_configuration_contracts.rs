use darkstore_concierge::{AppError, catalog::parse_turso_connection_configuration};

#[test]
fn test_req_tauri_015_rejects_absent_or_blank_turso_configuration() {
    let absent = parse_turso_connection_configuration(None, None);
    let blank_url = parse_turso_connection_configuration(Some("  "), Some("token"));
    let blank_token =
        parse_turso_connection_configuration(Some("libsql://fixture.turso.io"), Some(""));

    assert!(matches!(absent, Err(AppError::InventoryUnavailable)));
    assert!(matches!(blank_url, Err(AppError::InventoryUnavailable)));
    assert!(matches!(blank_token, Err(AppError::InventoryUnavailable)));
}
