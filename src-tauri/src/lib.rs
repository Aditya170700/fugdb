mod commands;
mod drivers;
mod error;
mod models;
mod schema;
mod qa;
mod state;
mod transfer;
mod ai;
mod keyring;
mod ssh;

use commands::{
    connection::{
        connect_database, disconnect_database, test_connection, test_ssh_tunnel,
        save_keyring_credential, get_keyring_credential, delete_keyring_credential
    },
    mock_data::{
        execute_mock_batch_insert, generate_mock_batch, generate_mock_sql_script,
        inspect_table_mock_config, preview_mock_rows,
    },
    query::{execute_query, explain_query, begin_transaction, commit_transaction, rollback_transaction},
    schema::{export_data_dictionary_file, fetch_schema_tree, generate_data_dictionary, generate_erd_metadata, print_data_dictionary, save_image_file, compare_schemas, apply_migration_script},
    transfer::{cancel_transfer_job, inspect_file, start_db_to_db_job, start_export_job, start_import_job},
    ai::{generate_sql_from_prompt, test_ai_connection, list_ollama_models, fix_sql_error, optimize_query_explain},
    monitor::{get_server_processes, kill_server_process, cancel_server_query},
};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            test_connection,
            test_ssh_tunnel,
            connect_database,
            disconnect_database,
            save_keyring_credential,
            get_keyring_credential,
            delete_keyring_credential,
            execute_query,
            begin_transaction,
            commit_transaction,
            rollback_transaction,
            fetch_schema_tree,
            generate_erd_metadata,
            generate_data_dictionary,
            export_data_dictionary_file,
            print_data_dictionary,
            save_image_file,
            compare_schemas,
            apply_migration_script,
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
            fix_sql_error,
            explain_query,
            optimize_query_explain,
            get_server_processes,
            kill_server_process,
            cancel_server_query
        ])
        .run(tauri::generate_context!())
        .expect("error while running FugDB application");
}
