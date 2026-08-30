use thiserror::Error;

/// core 层错误。前四个变体的文案与原 GUI 后端（src-tauri）返回给前端的
/// 错误字符串保持一致，GUI 行为不变。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CoreError {
    #[error("spe文件损坏")]
    SpeFileCorrupted,
    #[error("未知的文件格式")]
    UnknownFileFormat,
    #[error("尚不支持打开文件夹")]
    FolderNotSupported,
    #[error("未能解析到有效数据")]
    NoValidData,
    /// eV 模式与 k 轴模式依赖波长数据（GUI 中相应控件在无波长时不可达），
    /// 作为公开库在此显式报错而非 panic。
    #[error("eV 模式或 k 轴模式需要波长数据")]
    WavelengthRequired,
    #[error("数据形状与 width/height 不符")]
    ShapeMismatch,
    #[error("插值计算失败")]
    InterpolationFailed,
}
