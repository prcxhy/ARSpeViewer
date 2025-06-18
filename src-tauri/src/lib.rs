// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod data_extract;
mod heatmap_stretch;

use data_extract::*;
use heatmap_stretch::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![open_file, stretch])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
