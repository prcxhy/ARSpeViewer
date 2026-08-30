use serde::{Deserialize, Serialize};

/// 光谱数据模型。字段名与 GUI 前后端之间的 JSON 契约保持一致，勿随意改名。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SpeData {
    pub min_max: Vec<Vec<f64>>,
    pub frame: Vec<Vec<f64>>,
    pub width: usize,
    pub height: usize,
    pub wavelength: Option<Vec<f64>>,
    pub detector_angle_cal: f64,
    pub focal_length_cal: f64,
    pub inclusion_angle_cal: f64,
    pub detector_angle_exp: f64,
    pub focal_length_exp: f64,
    pub inclusion_angle_exp: f64,
}

impl SpeData {
    pub(crate) fn calc_maxs_mins(&mut self) {
        self.min_max = self
            .frame
            .iter()
            .map(|one_frame| {
                let mut max: f64 = one_frame[0];
                let mut min: f64 = one_frame[0];
                for val in one_frame {
                    if val > &max {
                        max = *val
                    }
                    if val < &min {
                        min = *val
                    }
                }
                vec![min, max]
            })
            .collect();
    }
}
