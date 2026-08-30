use darkstore_concierge::{AppError, commands::create_runtime_app_services};

#[tokio::test]
async fn test_req_tauri_001_and_012_command_services_keep_session_memory_only() {
    let services = create_runtime_app_services();

    let invalid = services.configure_session_openai_key("invalid").await;
    assert_eq!(invalid, Err(AppError::InvalidApiKey));

    let configured = services
        .configure_session_openai_key("sk-test-key-that-is-never-sent")
        .await
        .expect("syntactic key validation unlocks the in-memory command service");
    assert!(configured.concierge_enabled);

    services.clear_session_secret_state().await;
    let after_clear = services.load_initial_product_trio().await;
    assert_eq!(after_clear, Err(AppError::SessionUnavailable));
}
