use crate::data_extract::SpeData;
use ndarray::Array;
use ninterp::{
    prelude::{Extrapolate, Interp2D, Interpolator},
    strategy,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

const H: f64 = 6.62607015e-34;
const C0: f64 = 299792458.0;
const E: f64 = 1.602176634e-19;
const CONST_1240: f64 = H * C0 * 1e9 / E;

#[tauri::command]
pub fn stretch(
    spe_str: String,
    ev_mode: bool,
    x_mode: String,
    tan_min: f64,
    tan_max: f64,
) -> String {
    let mut spe_data: SpeData = serde_json::from_str(&spe_str).unwrap();

    let (width, height) = (spe_data.width, spe_data.height);

    let (y_grid, lambda_min, lambda_max) = if let Some(wavelength) = spe_data.wavelength {
        (
            Array::from_vec(wavelength.clone()),
            Some(wavelength[0]),
            Some(wavelength[width - 1]),
        )
    } else {
        (Array::linspace(0., (width - 1) as f64, width), None, None)
    };

    let x_grid = Array::linspace(tan_min, tan_max, height);

    let y_iter = if ev_mode {
        let (energy_min, energy_max) = (
            CONST_1240 / lambda_max.unwrap(),
            CONST_1240 / lambda_min.unwrap(),
        );
        Array::linspace(energy_max, energy_min, width).map(|energy| CONST_1240 / energy)
    } else {
        y_grid.clone()
    };

    let x_iter = match x_mode.as_str() {
        "angle" => Array::linspace(tan_min.atan(), tan_max.atan(), height)
            .map(|angle| angle.tan())
            .into_iter(),
        "k" => Array::linspace(tan_min / lambda_max.unwrap(), tan_max / lambda_max.unwrap(), height).into_iter(),
        _ => Array::linspace(tan_min, tan_max, height).into_iter(),
    };

    let mut points: Vec<Vec<f64>> = Vec::new();

    for x in x_iter {
        for y in y_iter.iter() {
            match x_mode.as_str() {
                "k" => {
                    points.push(vec![x * y, *y]);
                }
                _ => {
                    points.push(vec![x, *y]);
                }
            }
        }
    }

    let new_frames = spe_data
        .frame
        .iter()
        .map(|one_frame| {
            let data_raw = Array::from_shape_vec((height, width), one_frame.clone()).unwrap();

            let interp = Interp2D::new(
                x_grid.clone(),
                y_grid.clone(),
                data_raw,
                strategy::Linear,
                Extrapolate::Clamp,
            )
            .unwrap();

            points
                .par_iter()
                .map(|point: &Vec<f64>| interp.interpolate(&point).unwrap())
                .collect::<Vec<f64>>()
        })
        .collect::<Vec<Vec<f64>>>();

    spe_data.frame = new_frames;
    spe_data.wavelength = if lambda_max.is_some() {
        Some(y_iter.iter().map(|y| *y).collect())
    } else {
        None
    };

    serde_json::to_string(&spe_data).unwrap()
}
