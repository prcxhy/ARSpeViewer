use quick_xml::{events::Event, name::QName, reader::Reader};
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
struct SpeData {
    min_max: Vec<Vec<i16>>,
    frame: Vec<Vec<i16>>,
    width: usize,
    height: usize,
    wavelength: Vec<f64>,
    detector_angle_cal: f64,
    focal_length_cal: f64,
    inclusion_angle_cal: f64,
    detector_angle_exp: f64,
    focal_length_exp: f64,
    inclusion_angle_exp: f64,
}

impl SpeData {
    fn init() -> Self {
        SpeData {
            min_max: vec![vec![0]],
            frame: Vec::new(),
            width: 0,
            height: 0,
            wavelength: Vec::new(),
            detector_angle_cal: 0.0,
            focal_length_cal: 0.0,
            inclusion_angle_cal: 0.0,
            detector_angle_exp: 0.0,
            focal_length_exp: 0.0,
            inclusion_angle_exp: 0.0,
        }
    }
    fn calc_maxs_mins(&mut self) {
        self.min_max = self
            .frame
            .iter()
            .map(|one_frame| {
                let mut max: i16 = one_frame[0];
                let mut min: i16 = one_frame[0];
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

#[tauri::command]
pub fn open_spe(path: String) -> Result<String, String> {
    let data = fs::read(path);
    let mut spe_data = SpeData::init();

    if let Ok(data_vec) = data {
        let string_try = String::from_utf8_lossy(&data_vec[4100..]).to_string();

        let xml_start = string_try.find("<SpeFormat").unwrap();

        let mut reader = Reader::from_str(&string_try[xml_start..]);

        let mut frame_count: usize = 1;
        let mut frame_length: usize = 0;

        let mut detector_angle: Vec<f64> = Vec::new();
        let mut focal_length: Vec<f64> = Vec::new();
        let mut inclusion_angle: Vec<f64> = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"DataBlock" => {
                        if e.try_get_attribute(QName(b"type"))
                            .unwrap()
                            .unwrap()
                            .value
                            .as_ref()
                            == b"Frame"
                        {
                            let count = e.try_get_attribute(QName(b"count")).unwrap().unwrap();
                            frame_count = count.unescape_value().unwrap().parse().unwrap_or(1);
                            let count = e.try_get_attribute(QName(b"stride")).unwrap().unwrap();
                            frame_length = count.unescape_value().unwrap().parse().unwrap();
                        }
                    }
                    b"Wavelength" => {
                        if e.try_get_attribute(QName(b"type")).unwrap().is_none() {
                            if let Event::Text(text) = reader.read_event().unwrap() {
                                text.unescape().unwrap().split(',').for_each(|val| {
                                    spe_data.wavelength.push(val.parse::<f64>().unwrap())
                                });
                            }
                        }
                    }
                    b"DetectorAngle" => {
                        if let Event::Text(text) = reader.read_event().unwrap() {
                            detector_angle.push(text.unescape().unwrap().parse::<f64>().unwrap());
                        }
                    }
                    b"FocalLength" => {
                        if let Event::Text(text) = reader.read_event().unwrap() {
                            focal_length.push(text.unescape().unwrap().parse::<f64>().unwrap());
                        }
                    }
                    b"InclusionAngle" => {
                        if let Event::Text(text) = reader.read_event().unwrap() {
                            inclusion_angle.push(text.unescape().unwrap().parse::<f64>().unwrap());
                        }
                    }
                    _ => (),
                },
                Ok(Event::Empty(e)) => {
                    if e.name().as_ref() == b"DataBlock" {
                        if e.try_get_attribute(QName(b"type"))
                            .unwrap()
                            .unwrap()
                            .value
                            .as_ref()
                            == b"Region"
                        {
                            let count = e.try_get_attribute(QName(b"width")).unwrap().unwrap();
                            spe_data.width = count.unescape_value().unwrap().parse().unwrap();
                            let count = e.try_get_attribute(QName(b"height")).unwrap().unwrap();
                            spe_data.height = count.unescape_value().unwrap().parse().unwrap();
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => (),
            }
        }

        spe_data.detector_angle_cal = detector_angle[0];
        spe_data.focal_length_cal = focal_length[0];
        spe_data.inclusion_angle_cal = inclusion_angle[0];
        spe_data.detector_angle_exp = detector_angle[1];
        spe_data.focal_length_exp = focal_length[1];
        spe_data.inclusion_angle_exp = inclusion_angle[1];

        spe_data.frame = data_vec[4100..(4100 + frame_length * frame_count)]
            .chunks(frame_length)
            .map(|frame| {
                frame
                    .chunks(2)
                    .map(|piece| i16::from_le_bytes([piece[0], piece[1]]))
                    .collect::<Vec<i16>>()
            })
            .collect::<Vec<Vec<i16>>>();

        spe_data.calc_maxs_mins();
    } else {
        return Err(String::from("spe文件打开失败"));
    };
    let string = serde_json::to_string(&spe_data).unwrap();

    Ok(string)
}
