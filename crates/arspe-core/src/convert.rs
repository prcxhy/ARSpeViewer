//! 物理常数与坐标换算。
//!
//! 自前端 `ParametersConvert.ts` / `DataViewer.ts` 平移的纯数学部分：
//! 波长↔能量、tanθ↔角度↔波矢 k、纵/横轴区间与索引映射、跨文件区间兼容性。
//! 前端 TS 版本保留（供交互层低延迟使用），一致性由 golden 测试锁定。

use crate::model::SpeData;
use std::f64::consts::PI;

pub const H: f64 = 6.62607015e-34;
pub const C0: f64 = 299792458.0;
pub const E: f64 = 1.602176634e-19;
/// λ(nm) × E(eV) ≈ 1239.84
pub const CONST_1240: f64 = H * C0 * 1e9 / E;

// ---- 基本换算 ----

pub fn lambda_to_energy(lambda_nm: f64) -> f64 {
    CONST_1240 / lambda_nm
}

pub fn energy_to_lambda(energy_ev: f64) -> f64 {
    CONST_1240 / energy_ev
}

pub fn tan_to_angle(tan_theta: f64) -> f64 {
    tan_theta.atan() / PI * 180.0
}

pub fn angle_to_tan(angle_deg: f64) -> f64 {
    (angle_deg * PI / 180.0).tan()
}

/// k (μm⁻¹) = 2π·tanθ / λ；`lambda_nm` 为参与换算的波长（GUI 语义中取波长轴末端）。
pub fn tan_to_k(tan_theta: f64, lambda_nm: f64) -> f64 {
    2.0 * PI * tan_theta / lambda_nm * 1000.0
}

pub fn k_to_tan(k_um_inv: f64, lambda_nm: f64) -> f64 {
    k_um_inv * lambda_nm / (1000.0 * 2.0 * PI)
}

// ---- 纵轴区间（波长/能量） ----

/// 纵轴区间。以波长为内部表示，能量视图按 CONST_1240 换算。
#[derive(Debug, Clone, PartialEq)]
pub struct YRange {
    min_lambda: f64,
    max_lambda: f64,
}

impl YRange {
    pub fn new(min_lambda: f64, max_lambda: f64) -> Self {
        YRange {
            min_lambda,
            max_lambda,
        }
    }

    /// 从波长数组构造（取首尾），对应 TS `new YRange(wavelength)`。
    pub fn from_wavelength(wavelength: &[f64]) -> Self {
        YRange::new(wavelength[0], wavelength[wavelength.len() - 1])
    }

    pub fn min_lambda(&self) -> f64 {
        self.min_lambda
    }
    pub fn max_lambda(&self) -> f64 {
        self.max_lambda
    }
    pub fn set_min_lambda(&mut self, val: f64) {
        self.min_lambda = val;
    }
    pub fn set_max_lambda(&mut self, val: f64) {
        self.max_lambda = val;
    }
    pub fn min_energy(&self) -> f64 {
        CONST_1240 / self.max_lambda
    }
    pub fn max_energy(&self) -> f64 {
        CONST_1240 / self.min_lambda
    }
    pub fn set_min_energy(&mut self, val: f64) {
        self.max_lambda = CONST_1240 / val;
    }
    pub fn set_max_energy(&mut self, val: f64) {
        self.min_lambda = CONST_1240 / val;
    }
    pub fn band_width(&self) -> f64 {
        self.max_lambda - self.min_lambda
    }
    pub fn band_gap(&self) -> f64 {
        self.max_energy() - self.min_energy()
    }

    /// 本区间映射到 `range` 区间上的索引范围（长度 `length`）。
    pub fn index_range_of_lambda_in(&self, range: &YRange, length: f64) -> [i64; 2] {
        [
            ((self.min_lambda - range.min_lambda) / range.band_width() * length).round() as i64,
            ((self.max_lambda - range.min_lambda) / range.band_width() * length).round() as i64,
        ]
    }

    pub fn index_range_of_energy_in(&self, range: &YRange, length: f64) -> [i64; 2] {
        [
            ((range.max_energy() - self.max_energy()) / range.band_gap() * length).round() as i64,
            ((range.max_energy() - self.min_energy()) / range.band_gap() * length).round() as i64,
        ]
    }
}

// ---- 横轴区间（tanθ/角度/k） ----

/// 横轴区间。以 tanθ 为内部表示，角度与 k 视图按公式换算。
#[derive(Debug, Clone, PartialEq)]
pub struct XRange {
    min_tan: f64,
    max_tan: f64,
    lambda: f64,
}

impl XRange {
    /// 由数值孔径 NA 构造全域 ±tan(asin(NA))，对应 TS `new XRange(NA, lambda)`。
    pub fn new(na: f64, lambda: f64) -> Self {
        let tan = na.asin().tan();
        XRange {
            min_tan: -tan,
            max_tan: tan,
            lambda,
        }
    }

    /// 绑定模式：区间 `[from, to]`（长度基准 `length`）映射回完整 ±tan(asin(NA))，
    /// 对应 TS `new XRange(NA, lambda, [from, to, length])`。
    pub fn bound(na: f64, lambda: f64, from: f64, to: f64, length: f64) -> Self {
        let tan = na.asin().tan();
        let min_tan = -tan - 2.0 * tan / (to - from) * from;
        let max_tan = min_tan + 2.0 * tan / (to - from) * length;
        XRange {
            min_tan,
            max_tan,
            lambda,
        }
    }

    pub fn min_tan(&self) -> f64 {
        self.min_tan
    }
    pub fn max_tan(&self) -> f64 {
        self.max_tan
    }
    pub fn set_min_tan(&mut self, val: f64) {
        self.min_tan = val;
    }
    pub fn set_max_tan(&mut self, val: f64) {
        self.max_tan = val;
    }
    pub fn min_angle(&self) -> f64 {
        tan_to_angle(self.min_tan)
    }
    pub fn max_angle(&self) -> f64 {
        tan_to_angle(self.max_tan)
    }
    pub fn set_min_angle(&mut self, val: f64) {
        self.min_tan = angle_to_tan(val);
    }
    pub fn set_max_angle(&mut self, val: f64) {
        self.max_tan = angle_to_tan(val);
    }
    pub fn min_k(&self) -> f64 {
        tan_to_k(self.min_tan, self.lambda)
    }
    pub fn max_k(&self) -> f64 {
        tan_to_k(self.max_tan, self.lambda)
    }
    pub fn set_min_k(&mut self, val: f64) {
        self.min_tan = k_to_tan(val, self.lambda);
    }
    pub fn set_max_k(&mut self, val: f64) {
        self.max_tan = k_to_tan(val, self.lambda);
    }

    pub fn index_range_of_tan_in(&self, range: &XRange, length: f64) -> [i64; 2] {
        [
            ((self.min_tan - range.min_tan) / (range.max_tan - range.min_tan) * length).round()
                as i64,
            ((self.max_tan - range.min_tan) / (range.max_tan - range.min_tan) * length).round()
                as i64,
        ]
    }

    pub fn index_range_of_angle_in(&self, range: &XRange, length: f64) -> [i64; 2] {
        [
            ((self.min_angle() - range.min_angle()) / (range.max_angle() - range.min_angle())
                * length)
                .round() as i64,
            ((self.max_angle() - range.min_angle()) / (range.max_angle() - range.min_angle())
                * length)
                .round() as i64,
        ]
    }

    pub fn index_range_of_k_in(&self, range: &XRange, length: f64) -> [i64; 2] {
        [
            ((self.min_k() - range.min_k()) / (range.max_k() - range.min_k()) * length).round()
                as i64,
            ((self.max_k() - range.min_k()) / (range.max_k() - range.min_k()) * length).round()
                as i64,
        ]
    }
}

// ---- 跨文件区间兼容性 ----

/// 兼容性判定结果，对应前端 `compatible` 返回的 info 对象
/// （`y_compatible`/`x_compatible` 即 TS 中的 `yCompate`/`xCompate`）。
#[derive(Debug, Clone, PartialEq)]
pub struct CompatibleInfo {
    pub y_compatible: bool,
    pub y_min: f64,
    pub y_max: f64,
    pub x_compatible: bool,
    pub x_min_index: usize,
    pub x_max_index: usize,
}

/// 切换文件后判断旧视图区间能否延续到新数据上，对应前端 `compatible`。
pub fn compatible(
    new_spe: &SpeData,
    old_spe: &SpeData,
    x_min_index: usize,
    x_max_index: usize,
    y_min_input: f64,
    y_max_input: f64,
) -> CompatibleInfo {
    let mut info = CompatibleInfo {
        y_compatible: false,
        y_min: 0.0,
        y_max: 0.0,
        x_compatible: false,
        x_min_index: 0,
        x_max_index: new_spe.height - 1,
    };

    let inputs_are_ev = old_spe.wavelength.is_some() && y_min_input <= CONST_1240.sqrt();
    let lambda_min = if inputs_are_ev {
        CONST_1240 / y_max_input
    } else {
        y_min_input
    };
    let lambda_max = if inputs_are_ev {
        CONST_1240 / y_min_input
    } else {
        y_max_input
    };

    let new_has = new_spe.wavelength.is_some();
    let old_has = old_spe.wavelength.is_some();
    let new_last = new_has
        .then(|| new_spe.wavelength.as_ref().unwrap()[new_spe.width - 1])
        .unwrap_or(f64::NAN);
    let new_first = new_has
        .then(|| new_spe.wavelength.as_ref().unwrap()[0])
        .unwrap_or(f64::NAN);

    if new_has && old_has && !(lambda_min >= new_last || lambda_max <= new_first) {
        let old_wl = old_spe.wavelength.as_ref().unwrap();
        let bandwidth = new_last - new_first;
        if (new_first - old_wl[0]).abs() <= bandwidth / 10.0
            && (new_last - old_wl[old_spe.width - 1]).abs() <= bandwidth / 10.0
        {
            info.y_compatible = true;
            info.y_min = lambda_min.max(new_first);
            info.y_max = lambda_max.min(new_last);
        } else {
            info.y_min = new_first;
            info.y_max = new_last;
        }
    } else if !new_has && !old_has && new_spe.width == old_spe.width {
        info.y_compatible = true;
        info.y_min = lambda_min;
        info.y_max = lambda_max;
    } else if !new_has {
        info.y_min = 0.0;
        info.y_max = (new_spe.width - 1) as f64;
    } else {
        info.y_min = new_first;
        info.y_max = new_last;
    }

    if new_spe.height == old_spe.height {
        info.x_compatible = true;
        info.x_min_index = x_min_index;
        info.x_max_index = x_max_index;
    }
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    #[test]
    fn const_1240_value() {
        assert!((CONST_1240 - 1239.8419843320026).abs() < 1e-9);
    }

    #[test]
    fn lambda_energy_inverse() {
        for nm in [400.0, 532.0, 850.0] {
            assert!((energy_to_lambda(lambda_to_energy(nm)) - nm).abs() < EPS);
        }
    }

    #[test]
    fn tan_angle_inverse() {
        for deg in [5.0, 15.0, 30.0, 44.0] {
            assert!((tan_to_angle(angle_to_tan(deg)) - deg).abs() < EPS);
        }
    }

    #[test]
    fn tan_k_inverse() {
        let lambda = 850.0;
        for t in [-0.6, 0.0, 0.35] {
            assert!((k_to_tan(tan_to_k(t, lambda), lambda) - t).abs() < 1e-9);
        }
    }

    #[test]
    fn yrange_index_mapping_lambda() {
        let full = YRange::new(400.0, 1000.0);
        // 子区间 [400, 700] → [0, 50]
        assert_eq!(
            YRange::new(400.0, 700.0).index_range_of_lambda_in(&full, 100.0),
            [0, 50]
        );
    }

    #[test]
    fn yrange_index_mapping_energy() {
        let full = YRange::new(400.0, 1000.0);
        // 能量视角全域
        assert_eq!(
            full.index_range_of_energy_in(&full, 100.0),
            [0, 100]
        );
        // [700, 1000] nm 是能量低段 → 能量轴反向，映射到索引 [71, 100]
        assert_eq!(
            YRange::new(700.0, 1000.0).index_range_of_energy_in(&full, 100.0),
            [71, 100]
        );
    }

    #[test]
    fn xrange_na_and_bound() {
        let full = XRange::new(0.5, 850.0);
        let tan_na = 0.5f64.asin().tan();
        assert!((full.max_tan() - tan_na).abs() < EPS);
        assert!((full.max_angle() - 30.0).abs() < EPS);

        let bound = XRange::bound(0.5, 850.0, 0.0, 50.0, 100.0);
        assert!((bound.min_tan() - (-tan_na)).abs() < EPS);
        // TS 公式：maxTan = minTan + 2·tan/(to-from)·length = -tan + 4·tan = 3·tan
        assert!((bound.max_tan() - 3.0 * tan_na).abs() < EPS);
        assert_eq!(bound.index_range_of_tan_in(&full, 100.0), [0, 200]);
    }

    #[test]
    fn compatible_same_shape_full_overlap() {
        let mut a = SpeData::default();
        a.width = 3;
        a.height = 2;
        a.wavelength = Some(vec![400.0, 500.0, 600.0]);
        a.frame = vec![vec![0.0; 6]];
        let b = a.clone();
        let info = compatible(&a, &b, 0, 1, 400.0, 600.0);
        assert!(info.y_compatible);
        assert!((info.y_min - 400.0).abs() < EPS);
        assert!((info.y_max - 600.0).abs() < EPS);
        assert!(info.x_compatible);
        assert_eq!((info.x_min_index, info.x_max_index), (0, 1));
    }
}
