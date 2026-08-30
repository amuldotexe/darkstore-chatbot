pub mod catalog;
pub mod commands;
pub mod error;
pub mod model;
pub mod workflow;

pub use error::AppError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_desktop_concierge() {
    let app_result = tauri::Builder::default()
        .manage(commands::create_runtime_app_services())
        .invoke_handler(tauri::generate_handler![
            commands::configure_session_openai_key,
            commands::load_initial_product_trio,
            commands::search_portfolio_products_page,
            commands::select_product_chat_context,
            commands::update_product_variant_selection,
            commands::add_validated_variant_cart,
            commands::clear_session_secret_state,
        ])
        .run(tauri::generate_context!());
    if let Err(error) = app_result {
        eprintln!("desktop runtime terminated: {error}");
    }
}
