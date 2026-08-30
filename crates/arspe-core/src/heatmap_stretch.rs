//! 热图坐标轴插值拉伸（对应 GUI 的 `stretch` 命令）。
//!
//! 逻辑自 src-tauri 原样平移：按 eV 模式重建波长轴、按 x 轴模式重建横轴网格，
//! 以 2D 线性插值（Extrapolate::Clamp）将每帧重采样到新网格。

use crate::convert::CONST_1240;
use crate::error::CoreError;
use crate::model::SpeData;
use ndarray::Array;
use ninterp::{
    prelude::{Extrapolate, Interp2D},
    strategy,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// 对光谱做坐标轴拉伸重采样。
///
/// - `ev_mode`：纵轴按能量（eV）等间隔重建（波长轴随之非线性化）；
/// - `x_mode`：`"tan"` / `"angle"` / `"k"`，横轴网格重建方式；
/// - `tan_min` / `tan_max`：tanθ 范围（角度与 k 模式据此换算）。
///
/// eV 与 k 模式需要波长数据，缺失时返回 [`CoreError::WavelengthRequired`]。
pub fn stretch(
    spe: &SpeData,
    ev_mode: bool,
    x_mode: &str,
    tan_min: f64,
    tan_max: f64,
) -> Result<SpeData, CoreError> {
    let mut spe_data = spe.clone();

    let (width, height) = (spe_data.width, spe_data.height);

    let x_grid = Array::linspace(tan_min, tan_max, height);

    let (y_grid, lambda_min, lambda_max) = if let Some(wavelength) = &spe_data.wavelength {
        (
            Array::from_vec(wavelength.clone()),
            Some(wavelength[0]),
            Some(wavelength[width - 1]),
        )
    } else {
        (Array::linspace(0., (width - 1) as f64, width), None, None)
    };

    let y_iter = if ev_mode {
        let (energy_min, energy_max) = (
            CONST_1240 / lambda_max.ok_or(CoreError::WavelengthRequired)?,
            CONST_1240 / lambda_min.ok_or(CoreError::WavelengthRequired)?,
        );
        Array::linspace(energy_max, energy_min, width).map(|energy| CONST_1240 / energy)
    } else {
        y_grid.clone()
    };

    let x_iter = match x_mode {
        "angle" => Array::linspace(tan_min.atan(), tan_max.atan(), height)
            .map(|angle| angle.tan())
            .into_iter(),
        "k" => Array::linspace(
            tan_min / lambda_max.ok_or(CoreError::WavelengthRequired)?,
            tan_max / lambda_max.ok_or(CoreError::WavelengthRequired)?,
            height,
        )
        .into_iter(),
        _ => Array::linspace(tan_min, tan_max, height).into_iter(),
    };

    let mut points: Vec<[f64; 2]> = Vec::new();

    for x in x_iter {
        for y in y_iter.iter() {
            match x_mode {
                "k" => {
                    points.push([x * y, *y]);
                }
                _ => {
                    points.push([x, *y]);
                }
            }
        }
    }

    let new_frames = spe_data
        .frame
        .iter()
        .map(|one_frame| {
            let data_raw = Array::from_shape_vec((height, width), one_frame.clone())
                .map_err(|_| CoreError::ShapeMismatch)?;

            let interp = Interp2D::new(
                x_grid.clone(),
                y_grid.clone(),
                data_raw,
                strategy::Linear,
                Extrapolate::Clamp,
            )
            .map_err(|_| CoreError::InterpolationFailed)?;

            points
                .par_iter()
                .map(|point: &[f64; 2]| {
                    interp
                        .interpolate(point)
                        .map_err(|_| CoreError::InterpolationFailed)
                })
                .collect::<Result<Vec<f64>, CoreError>>()
        })
        .collect::<Result<Vec<Vec<f64>>, CoreError>>()?;

    spe_data.frame = new_frames;
    spe_data.wavelength = if lambda_max.is_some() {
        Some(y_iter.iter().map(|y| *y).collect())
    } else {
        None
    };

    Ok(spe_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_extract::parse_txt;

    fn make_row_major_spe(data_rows: usize, cols: usize) -> SpeData {
        let mut lines: Vec<String> = Vec::new();
        let wl: Vec<String> = (0..cols).map(|i| format!("{:.1}", 400.0 + i as f64)).collect();
        lines.push(wl.join("\t"));
        for r in 1..=data_rows {
            let row: Vec<String> = (0..cols)
                .map(|i| format!("{:.1}", r as f64 * 1000.0 + i as f64))
                .collect();
            lines.push(row.join("\t"));
        }
        parse_txt(&lines.join("\n")).unwrap()
    }

    #[test]
    fn stretch_tan_full_range_is_identity() {
        let spe = make_row_major_spe(3, 101);
        // x 网格 [0,1,2] 与数据行索引重合 → 输出与输入逐值一致
        let out = stretch(&spe, false, "tan", 0.0, 2.0).unwrap();
        assert_eq!(out.frame, spe.frame);
        assert_eq!(out.wavelength, spe.wavelength);
        assert_eq!(out.min_max, spe.min_max);
    }

    #[test]
    fn stretch_tan_mode_is_x_identity_without_ev() {
        let spe = make_row_major_spe(3, 101);
        // tan 模式下 x 求值点与节点重合（真正的重采样在 eV 纵轴与 angle/k 横轴）
        let out = stretch(&spe, false, "tan", 0.0, 1.0).unwrap();
        assert_eq!(out.frame, spe.frame);
        assert_eq!(out.wavelength, spe.wavelength);
    }

    #[test]
    fn stretch_angle_mode_compresses_edges() {
        let spe = make_row_major_spe(3, 101);
        // x 网格节点 [0, 0.25, 0.5]；angle 模式的中间求值点 = tan(atan(0.5)/2)
        let out = stretch(&spe, false, "angle", 0.0, 0.5).unwrap();
        let w = (0.5f64.atan() / 2.0).tan() / 0.25; // ≈ 0.94427，落在节点 0 与 0.25 之间
        for j in 0..out.width {
            let expected =
                spe.frame[0][j] * (1.0 - w) + spe.frame[0][out.width + j] * w;
            assert!((out.frame[0][out.width + j] - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn stretch_ev_mode_remaps_wavelength() {
        let spe = make_row_major_spe(3, 101);
        let out = stretch(&spe, true, "tan", 0.0, 2.0).unwrap();
        let new_wl = out.wavelength.as_ref().unwrap();
        assert_eq!(new_wl.len(), 101);
        // eV 等间隔 → 波长轴被非线性重映射，首尾仍在原波长范围内
        assert!((new_wl[0] - 400.0).abs() < 1e-6);
        assert!((new_wl[100] - 500.0).abs() < 1e-6);
    }

    #[test]
    fn stretch_k_mode_requires_wavelength() {
        let spe = SpeData {
            width: 3,
            height: 2,
            frame: vec![vec![1.0; 6]],
            ..Default::default()
        };
        assert_eq!(
            stretch(&spe, false, "k", -1.0, 1.0),
            Err(CoreError::WavelengthRequired)
        );
        assert_eq!(
            stretch(&spe, true, "tan", -1.0, 1.0),
            Err(CoreError::WavelengthRequired)
        );
        // tan 模式无波长也可运行（网格 [0,2] 合法）
        let out = stretch(&spe, false, "tan", 0.0, 2.0).unwrap();
        assert_eq!(out.frame.len(), 1);
    }
}
