//! ARSpeViewer 核心库：光谱数据解析、坐标换算与插值拉伸。
//!
//! 该 crate 不依赖 Tauri，被 GUI（src-tauri）与 Python 绑定（arspe-py）共同引用，
//! 是数据处理口径的单一事实源。

pub mod convert;
mod data_extract;
mod error;
mod heatmap_stretch;
mod model;

pub use convert::{
    compatible, energy_to_lambda, k_to_tan, lambda_to_energy, tan_to_angle, tan_to_k,
    angle_to_tan, CompatibleInfo, XRange, YRange, CONST_1240,
};
pub use data_extract::{open_path, parse_spe, parse_txt};
pub use error::CoreError;
pub use heatmap_stretch::stretch;
pub use model::SpeData;
