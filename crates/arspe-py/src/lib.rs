//! arspe-py：ARSpeViewer 核心库（arspe-core）的 Python 绑定。
//!
//! 为 Agent / 自动化脚本提供无 GUI 的数据提取与换算能力：
//! 解析 `.spe` / `.txt` / `.csv` / `.asc`、坐标轴插值拉伸、numpy 数组输出与导出。

use std::path::PathBuf;

use arspe_core::{
    CoreError, SpeData as CoreSpeData, XRange as CoreXRange, YRange as CoreYRange,
};
use numpy::{PyArray1, PyArray2, PyArray3, PyArrayMethods};
use pyo3::{
    PyTypeInfo,
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::PyDict,
};

// ---- 异常族 ----

create_exception!(
    arspe_py,
    ArspeError,
    PyException,
    "ARSpeViewer 数据处理错误基类"
);
create_exception!(
    arspe_py,
    ParseError,
    ArspeError,
    "数据文件解析失败（文件损坏或内容无效）"
);
create_exception!(
    arspe_py,
    UnsupportedFormatError,
    ArspeError,
    "不支持的文件格式或路径类型"
);
create_exception!(
    arspe_py,
    StretchError,
    ArspeError,
    "坐标轴拉伸/换算失败（如 eV、k 模式缺少波长数据）"
);

fn to_pyerr(err: CoreError) -> PyErr {
    let msg = err.to_string();
    match err {
        CoreError::SpeFileCorrupted | CoreError::NoValidData => ParseError::new_err(msg),
        CoreError::UnknownFileFormat | CoreError::FolderNotSupported => {
            UnsupportedFormatError::new_err(msg)
        }
        CoreError::WavelengthRequired | CoreError::ShapeMismatch
        | CoreError::InterpolationFailed => StretchError::new_err(msg),
    }
}

// ---- 数据对象 ----

/// 光谱标定参数（来自 .spe 文件内嵌 XML）。
#[pyclass(name = "Calibration", module = "arspe_py")]
pub struct Calibration {
    #[pyo3(get)]
    detector_angle_cal: f64,
    #[pyo3(get)]
    focal_length_cal: f64,
    #[pyo3(get)]
    inclusion_angle_cal: f64,
    #[pyo3(get)]
    detector_angle_exp: f64,
    #[pyo3(get)]
    focal_length_exp: f64,
    #[pyo3(get)]
    inclusion_angle_exp: f64,
}

#[pymethods]
impl Calibration {
    fn __repr__(&self) -> String {
        format!(
            "Calibration(detector_angle_cal={}, focal_length_cal={}, inclusion_angle_cal={}, \
             detector_angle_exp={}, focal_length_exp={}, inclusion_angle_exp={})",
            self.detector_angle_cal,
            self.focal_length_cal,
            self.inclusion_angle_cal,
            self.detector_angle_exp,
            self.focal_length_exp,
            self.inclusion_angle_exp
        )
    }
}

/// 光谱数据对象（不可变）。
///
/// `frames` 形状为 `(n_frames, height, width)` 的 float64 数组：
/// `height` 为角度行（x 方向），`width` 为波长列（y 方向），
/// 与 GUI 前后端 JSON 契约的行列语义一致。
#[pyclass(name = "SpeData", module = "arspe_py")]
pub struct PySpeData {
    pub(crate) inner: CoreSpeData,
}

impl From<CoreSpeData> for PySpeData {
    fn from(inner: CoreSpeData) -> Self {
        PySpeData { inner }
    }
}

#[pymethods]
impl PySpeData {
    /// 波长列数（y 方向）。
    #[getter]
    fn width(&self) -> usize {
        self.inner.width
    }

    /// 角度行数（x 方向）。
    #[getter]
    fn height(&self) -> usize {
        self.inner.height
    }

    /// 帧数。
    #[getter]
    fn frame_count(&self) -> usize {
        self.inner.frame.len()
    }

    /// 全部帧数据，形状 `(n_frames, height, width)` 的 float64 数组。
    #[getter]
    fn frames<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray3<f64>>> {
        let n = self.inner.frame.len();
        let (h, w) = (self.inner.height, self.inner.width);
        let flat: Vec<f64> = self.inner.frame.iter().flatten().copied().collect();
        let flat = PyArray1::from_vec(py, flat);
        let out: Bound<'py, PyArray3<f64>> = flat.reshape([n, h, w])?;
        Ok(out)
    }

    /// 波长数组（nm），无波长数据时为 None。
    #[getter]
    fn wavelength<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyArray1<f64>>> {
        self.inner
            .wavelength
            .as_ref()
            .map(|wl| PyArray1::from_vec(py, wl.clone()))
    }

    /// 每帧 [min, max]，形状 `(n_frames, 2)`。
    #[getter]
    fn min_max<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        Ok(PyArray2::from_vec2(py, &self.inner.min_max)?)
    }

    /// 标定参数对象。
    #[getter]
    fn calibration(&self) -> Calibration {
        Calibration {
            detector_angle_cal: self.inner.detector_angle_cal,
            focal_length_cal: self.inner.focal_length_cal,
            inclusion_angle_cal: self.inner.inclusion_angle_cal,
            detector_angle_exp: self.inner.detector_angle_exp,
            focal_length_exp: self.inner.focal_length_exp,
            inclusion_angle_exp: self.inner.inclusion_angle_exp,
        }
    }

    /// 转为 dict（与 GUI 前后端 JSON 契约字段一致，帧数据为嵌套 list）。
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item("min_max", &self.inner.min_max)?;
        d.set_item("frame", &self.inner.frame)?;
        d.set_item("width", self.inner.width)?;
        d.set_item("height", self.inner.height)?;
        d.set_item("wavelength", &self.inner.wavelength)?;
        d.set_item("detector_angle_cal", self.inner.detector_angle_cal)?;
        d.set_item("focal_length_cal", self.inner.focal_length_cal)?;
        d.set_item("inclusion_angle_cal", self.inner.inclusion_angle_cal)?;
        d.set_item("detector_angle_exp", self.inner.detector_angle_exp)?;
        d.set_item("focal_length_exp", self.inner.focal_length_exp)?;
        d.set_item("inclusion_angle_exp", self.inner.inclusion_angle_exp)?;
        Ok(d)
    }

    /// 序列化为 JSON 字符串（字段与 GUI 契约一致）。
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|e| PyException::new_err(e.to_string()))
    }

    /// 导出单帧为制表符分隔矩阵（行 = 波长/行索引，列 = 角度行索引），
    /// 可直接粘贴到 Origin / Excel。
    #[pyo3(signature = (path, frame_index = None, header = None))]
    fn save_csv(
        &self,
        path: PathBuf,
        frame_index: Option<usize>,
        header: Option<bool>,
    ) -> PyResult<()> {
        let n = self.inner.frame.len();
        let fi = frame_index.unwrap_or(0);
        if fi >= n {
            return Err(PyValueError::new_err(format!(
                "frame_index {} 超出范围（共 {} 帧）",
                fi, n
            )));
        }
        let one_frame = &self.inner.frame[fi];
        let (h, w) = (self.inner.height, self.inner.width);

        let mut out = String::new();
        if header.unwrap_or(true) {
            out.push_str("y\\x");
            for x in 0..h {
                out.push('\t');
                out.push_str(&x.to_string());
            }
            out.push('\n');
        }
        for y in 0..w {
            match &self.inner.wavelength {
                Some(wl) => out.push_str(&wl[y].to_string()),
                None => out.push_str(&y.to_string()),
            }
            for x in 0..h {
                out.push('\t');
                out.push_str(&one_frame[x * w + y].to_string());
            }
            out.push('\n');
        }
        std::fs::write(path, out).map_err(|e| PyException::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "SpeData(frames={}, width={}, height={}, wavelength={})",
            self.inner.frame.len(),
            self.inner.width,
            self.inner.height,
            self.inner.wavelength.is_some()
        )
    }
}

// ---- 顶层函数 ----

#[pyfunction]
fn open_file(py: Python<'_>, path: PathBuf) -> PyResult<PySpeData> {
    py.detach(|| arspe_core::open_path(&path).map(PySpeData::from).map_err(to_pyerr))
}

/// 从字节流解析 `.spe` 二进制数据。
#[pyfunction]
fn parse_spe(py: Python<'_>, data: Vec<u8>) -> PyResult<PySpeData> {
    Ok(py.detach(|| PySpeData::from(arspe_core::parse_spe(&data))))
}

/// 从文本解析 `.txt` / `.csv` / `.asc` 数据。
#[pyfunction]
fn parse_text(py: Python<'_>, text: &str) -> PyResult<PySpeData> {
    py.detach(|| arspe_core::parse_txt(text).map(PySpeData::from).map_err(to_pyerr))
}

/// 坐标轴插值拉伸（与 GUI `stretch` 命令同源）。
///
/// - `ev_mode`：纵轴按能量（eV）等间隔重建；
/// - `x_mode`：`"tan"` / `"angle"` / `"k"`；
/// - `tan_min` / `tan_max`：tanθ 范围。
#[pyfunction]
fn stretch(
    py: Python<'_>,
    spe: &PySpeData,
    ev_mode: bool,
    x_mode: &str,
    tan_min: f64,
    tan_max: f64,
) -> PyResult<PySpeData> {
    py.detach(|| {
        arspe_core::stretch(&spe.inner, ev_mode, x_mode, tan_min, tan_max)
            .map(PySpeData::from)
            .map_err(to_pyerr)
    })
}

// ---- 坐标换算 ----

/// λ(nm) → E(eV)，E = 1239.84 / λ。
#[pyfunction]
fn lambda_to_energy(lambda_nm: f64) -> f64 {
    arspe_core::lambda_to_energy(lambda_nm)
}

/// E(eV) → λ(nm)。
#[pyfunction]
fn energy_to_lambda(energy_ev: f64) -> f64 {
    arspe_core::energy_to_lambda(energy_ev)
}

/// tanθ → 角度（°）。
#[pyfunction]
fn tan_to_angle(tan_theta: f64) -> f64 {
    arspe_core::tan_to_angle(tan_theta)
}

/// 角度（°）→ tanθ。
#[pyfunction]
fn angle_to_tan(angle_deg: f64) -> f64 {
    arspe_core::angle_to_tan(angle_deg)
}

/// tanθ → 波矢 k (μm⁻¹)，k = 2π·tanθ / λ。
#[pyfunction]
fn tan_to_k(tan_theta: f64, lambda_nm: f64) -> f64 {
    arspe_core::tan_to_k(tan_theta, lambda_nm)
}

/// 波矢 k (μm⁻¹) → tanθ。
#[pyfunction]
fn k_to_tan(k_um_inv: f64, lambda_nm: f64) -> f64 {
    arspe_core::k_to_tan(k_um_inv, lambda_nm)
}

// ---- 区间对象 ----

/// 纵轴区间（波长 nm / 能量 eV 双视图）。
#[pyclass(name = "YRange", module = "arspe_py")]
pub struct PyYRange {
    inner: CoreYRange,
}

#[pymethods]
impl PyYRange {
    #[new]
    fn new(min_lambda: f64, max_lambda: f64) -> Self {
        PyYRange {
            inner: CoreYRange::new(min_lambda, max_lambda),
        }
    }

    /// 从波长数组构造（取首尾），对应 TS `new YRange(wavelength)`。
    #[staticmethod]
    fn from_wavelength(wavelength: Vec<f64>) -> Self {
        PyYRange {
            inner: CoreYRange::from_wavelength(&wavelength),
        }
    }

    #[getter]
    fn min_lambda(&self) -> f64 {
        self.inner.min_lambda()
    }
    #[setter]
    fn set_min_lambda(&mut self, val: f64) {
        self.inner.set_min_lambda(val);
    }
    #[getter]
    fn max_lambda(&self) -> f64 {
        self.inner.max_lambda()
    }
    #[setter]
    fn set_max_lambda(&mut self, val: f64) {
        self.inner.set_max_lambda(val);
    }
    #[getter]
    fn min_energy(&self) -> f64 {
        self.inner.min_energy()
    }
    #[setter]
    fn set_min_energy(&mut self, val: f64) {
        self.inner.set_min_energy(val);
    }
    #[getter]
    fn max_energy(&self) -> f64 {
        self.inner.max_energy()
    }
    #[setter]
    fn set_max_energy(&mut self, val: f64) {
        self.inner.set_max_energy(val);
    }
    #[getter]
    fn band_width(&self) -> f64 {
        self.inner.band_width()
    }
    #[getter]
    fn band_gap(&self) -> f64 {
        self.inner.band_gap()
    }

    /// 本区间映射到 `range` 上的索引范围（能量视图）。
    fn index_range_of_energy_in(&self, range: &PyYRange, length: f64) -> (i64, i64) {
        let [a, b] = self.inner.index_range_of_energy_in(&range.inner, length);
        (a, b)
    }

    /// 本区间映射到 `range` 上的索引范围（波长视图）。
    fn index_range_of_lambda_in(&self, range: &PyYRange, length: f64) -> (i64, i64) {
        let [a, b] = self.inner.index_range_of_lambda_in(&range.inner, length);
        (a, b)
    }

    fn __repr__(&self) -> String {
        format!(
            "YRange(min_lambda={}, max_lambda={})",
            self.inner.min_lambda(),
            self.inner.max_lambda()
        )
    }
}

/// 横轴区间（tanθ / 角度° / 波矢 k 三视图）。
#[pyclass(name = "XRange", module = "arspe_py")]
pub struct PyXRange {
    inner: CoreXRange,
}

#[pymethods]
impl PyXRange {
    /// `binding = (from, to, length)` 为 TS 绑定模式的索引窗口。
    #[new]
    #[pyo3(signature = (na, lambda, binding = None))]
    fn new(na: f64, lambda: f64, binding: Option<(f64, f64, f64)>) -> Self {
        let inner = match binding {
            Some((from, to, length)) => CoreXRange::bound(na, lambda, from, to, length),
            None => CoreXRange::new(na, lambda),
        };
        PyXRange { inner }
    }

    #[getter]
    fn min_tan(&self) -> f64 {
        self.inner.min_tan()
    }
    #[setter]
    fn set_min_tan(&mut self, val: f64) {
        self.inner.set_min_tan(val);
    }
    #[getter]
    fn max_tan(&self) -> f64 {
        self.inner.max_tan()
    }
    #[setter]
    fn set_max_tan(&mut self, val: f64) {
        self.inner.set_max_tan(val);
    }
    #[getter]
    fn min_angle(&self) -> f64 {
        self.inner.min_angle()
    }
    #[setter]
    fn set_min_angle(&mut self, val: f64) {
        self.inner.set_min_angle(val);
    }
    #[getter]
    fn max_angle(&self) -> f64 {
        self.inner.max_angle()
    }
    #[setter]
    fn set_max_angle(&mut self, val: f64) {
        self.inner.set_max_angle(val);
    }
    #[getter]
    fn min_k(&self) -> f64 {
        self.inner.min_k()
    }
    #[setter]
    fn set_min_k(&mut self, val: f64) {
        self.inner.set_min_k(val);
    }
    #[getter]
    fn max_k(&self) -> f64 {
        self.inner.max_k()
    }
    #[setter]
    fn set_max_k(&mut self, val: f64) {
        self.inner.set_max_k(val);
    }

    fn index_range_of_tan_in(&self, range: &PyXRange, length: f64) -> (i64, i64) {
        let [a, b] = self.inner.index_range_of_tan_in(&range.inner, length);
        (a, b)
    }

    fn index_range_of_angle_in(&self, range: &PyXRange, length: f64) -> (i64, i64) {
        let [a, b] = self.inner.index_range_of_angle_in(&range.inner, length);
        (a, b)
    }

    fn index_range_of_k_in(&self, range: &PyXRange, length: f64) -> (i64, i64) {
        let [a, b] = self.inner.index_range_of_k_in(&range.inner, length);
        (a, b)
    }

    fn __repr__(&self) -> String {
        format!(
            "XRange(min_tan={}, max_tan={})",
            self.inner.min_tan(),
            self.inner.max_tan()
        )
    }
}

#[pymodule]
fn arspe_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySpeData>()?;
    m.add_class::<Calibration>()?;
    m.add_class::<PyYRange>()?;
    m.add_class::<PyXRange>()?;
    m.add_function(wrap_pyfunction!(open_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_spe, m)?)?;
    m.add_function(wrap_pyfunction!(parse_text, m)?)?;
    m.add_function(wrap_pyfunction!(stretch, m)?)?;
    m.add_function(wrap_pyfunction!(lambda_to_energy, m)?)?;
    m.add_function(wrap_pyfunction!(energy_to_lambda, m)?)?;
    m.add_function(wrap_pyfunction!(tan_to_angle, m)?)?;
    m.add_function(wrap_pyfunction!(angle_to_tan, m)?)?;
    m.add_function(wrap_pyfunction!(tan_to_k, m)?)?;
    m.add_function(wrap_pyfunction!(k_to_tan, m)?)?;
    m.add("ArspeError", ArspeError::type_object(m.py()))?;
    m.add("ParseError", ParseError::type_object(m.py()))?;
    m.add("UnsupportedFormatError", UnsupportedFormatError::type_object(m.py()))?;
    m.add("StretchError", StretchError::type_object(m.py()))?;
    m.add("CONST_1240", arspe_core::CONST_1240)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
