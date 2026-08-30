fn main() {
    // pyo3 0.29 起不再自动注入 macOS 扩展模块链接参数，直接用 cargo 构建时
    // 需按官方指南在 build script 中显式调用，否则 macOS 链接报 Python 符号未定义。
    // （maturin/setuptools-rust 会代为传入，此处对齐其行为；非 macOS 平台为 no-op。）
    pyo3_build_config::add_extension_module_link_args();
}
