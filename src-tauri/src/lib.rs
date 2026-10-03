mod commands;
mod drivers;
mod error;
mod models;
mod schema;
mod qa;
mod state;
mod transfer;
mod ai;

use commands::{
    connection::{connect_database, disconnect_database, test_connection},
    mock_data::{
        execute_mock_batch_insert, generate_mock_batch, generate_mock_sql_script,
        inspect_table_mock_config, preview_mock_rows,
    },
    query::execute_query,
    schema::{export_data_dictionary_file, fetch_schema_tree, generate_data_dictionary, generate_erd_metadata, print_data_dictionary, save_image_file},
    transfer::{cancel_transfer_job, inspect_file, start_db_to_db_job, start_export_job, start_import_job},
    ai::{generate_sql_from_prompt, test_ai_connection, list_ollama_models, fix_sql_error},
};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            test_connection,
            connect_database,
            disconnect_database,
            execute_query,
            fetch_schema_tree,
            generate_erd_metadata,
            generate_data_dictionary,
            export_data_dictionary_file,
            print_data_dictionary,
            save_image_file,
            inspect_table_mock_config,
            preview_mock_rows,
            generate_mock_sql_script,
            execute_mock_batch_insert,
            generate_mock_batch,
            inspect_file,
            start_export_job,
            start_import_job,
            start_db_to_db_job,
            cancel_transfer_job,
            generate_sql_from_prompt,
            test_ai_connection,
            list_ollama_models,
            fix_sql_error
        ])
        .run(tauri::generate_context!())
        .expect("error while running FugDB application");
}
