//! Tauri 命令薄封装：参数/返回值与前端契约保持不变。

use arspe_core::SpeData;
use std::path::Path;

#[tauri::command]
pub fn open_file(path: String) -> Result<String, String> {
    let spe = arspe_core::open_path(Path::new(&path)).map_err(|e| e.to_string())?;
    serde_json::to_string(&spe).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stretch(
    spe_str: String,
    ev_mode: bool,
    x_mode: String,
    tan_min: f64,
    tan_max: f64,
) -> Result<String, String> {
    let spe: SpeData = serde_json::from_str(&spe_str).map_err(|e| e.to_string())?;
    let stretched =
        arspe_core::stretch(&spe, ev_mode, &x_mode, tan_min, tan_max).map_err(|e| e.to_string())?;
    serde_json::to_string(&stretched).map_err(|e| e.to_string())
}
