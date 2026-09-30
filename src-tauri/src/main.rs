// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod drivers;
mod error;
mod models;
mod state;

use commands::{
    connection::{connect_database, disconnect_database, test_connection},
    mock_data::generate_mock_batch,
    query::execute_query,
    schema::{fetch_schema_tree, generate_erd_metadata},
};
use state::AppState;

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            test_connection,
            connect_database,
            disconnect_database,
            execute_query,
            fetch_schema_tree,
            generate_erd_metadata,
            generate_mock_batch
        ])
        .run(tauri::generate_context!())
        .expect("error while running FugDB application");
}
