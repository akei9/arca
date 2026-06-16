pub mod commands;
pub mod error;
pub mod state;

use commands::{
    create_entry, create_vault, delete_entry, generate_password, get_entry, get_settings,
    list_entries, lock_vault, search_entries, suggest_paths, unlock_vault, update_entry,
    update_settings,
};
use state::AppState;

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            unlock_vault,
            lock_vault,
            create_vault,
            list_entries,
            get_entry,
            create_entry,
            update_entry,
            delete_entry,
            search_entries,
            suggest_paths,
            generate_password,
            get_settings,
            update_settings
        ])
        .run(tauri::generate_context!())
}
