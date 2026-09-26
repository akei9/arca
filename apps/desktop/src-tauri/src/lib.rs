pub mod commands;
pub mod error;
pub mod preferences;
pub mod state;

use commands::{
    create_entry, create_vault, delete_entry, forget_remembered_vault, generate_password,
    get_entry, get_entry_revisions, get_remembered_vault, get_settings, list_entries, lock_vault,
    remember_current_vault, reveal_entry_password, reveal_entry_revision_password, search_entries,
    suggest_paths, unlock_vault, update_entry, update_settings,
};
use preferences::PreferencesState;
use state::AppState;

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::default())
        .manage(PreferencesState::default())
        .invoke_handler(tauri::generate_handler![
            unlock_vault,
            lock_vault,
            create_vault,
            get_remembered_vault,
            remember_current_vault,
            forget_remembered_vault,
            list_entries,
            get_entry,
            reveal_entry_password,
            get_entry_revisions,
            reveal_entry_revision_password,
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
