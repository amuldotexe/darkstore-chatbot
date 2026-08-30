use std::{fs, path::PathBuf};

use serde_json::Value;

fn read_tauri_project_file(relative_path: &str) -> String {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(project_root.join(relative_path))
        .expect("checked-in Tauri configuration file should be readable")
}

#[test]
fn test_req_tauri_011_scopes_main_window_to_core_only_and_self_csp() {
    let capability: Value =
        serde_json::from_str(&read_tauri_project_file("capabilities/default.json"))
            .expect("capability must stay valid JSON");
    let permissions = capability["permissions"]
        .as_array()
        .expect("capability permissions must be an array");
    let permission_names: Vec<&str> = permissions.iter().filter_map(Value::as_str).collect();
    let configuration: Value = serde_json::from_str(&read_tauri_project_file("tauri.conf.json"))
        .expect("Tauri configuration must stay valid JSON");
    let csp = configuration["app"]["security"]["csp"]
        .as_str()
        .expect("CSP must be a string");

    assert_eq!(capability["windows"], serde_json::json!(["main"]));
    assert_eq!(permission_names, vec!["core:default"]);
    assert!(csp.contains("default-src 'self'"));
    assert!(csp.contains("connect-src 'self'"));
    assert!(!csp.contains("https:"));
}
