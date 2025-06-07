use quick_xml::{events::Event, name::QName, reader::Reader};
use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Serialize)]
enum FrameVec {
    I16FRAME(Vec<i16>),
    I32FRAME(Vec<i32>),
    F32FRAME(Vec<f32>),
}

#[derive(Serialize)]
struct SpeData {
    min_max: Vec<FrameVec>,
    frame: Vec<FrameVec>,
    width: usize,
    height: usize,
    wavelength: Option<Vec<f64>>,
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
            min_max: Vec::new(),
            frame: Vec::new(),
            width: 0,
            height: 0,
            wavelength: None,
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
            .map(|one_frame| match one_frame {
                FrameVec::I16FRAME(vec) => {
                    let mut max: i16 = vec[0];
                    let mut min: i16 = vec[0];
                    for val in vec {
                        if val > &max {
                            max = *val
                        }
                        if val < &min {
                            min = *val
                        }
                    }
                    FrameVec::I16FRAME(vec![min, max])
                }
                FrameVec::I32FRAME(vec) => {
                    let mut max: i32 = vec[0];
                    let mut min: i32 = vec[0];
                    for val in vec {
                        if val > &max {
                            max = *val
                        }
                        if val < &min {
                            min = *val
                        }
                    }
                    FrameVec::I32FRAME(vec![min, max])
                }
                FrameVec::F32FRAME(vec) => {
                    let mut max: f32 = vec[0];
                    let mut min: f32 = vec[0];
                    for val in vec {
                        if val > &max {
                            max = *val
                        }
                        if val < &min {
                            min = *val
                        }
                    }
                    FrameVec::F32FRAME(vec![min, max])
                }
            })
            .collect();
    }
}

fn parse_spe(data_vec: Vec<u8>) -> String {
    let mut spe_data = SpeData::init();

    spe_data.width = u16::from_le_bytes([data_vec[6], data_vec[7]]) as usize;
    spe_data.height = u16::from_le_bytes([data_vec[18], data_vec[19]]) as usize;
    let frame_counts = data_vec[1446] as usize;
    let frame_size = spe_data.width * spe_data.height;

    let number_bytes: usize = if frame_size * frame_counts * 4 > data_vec.len() {
        2
    } else {
        4
    };

    let test_f32 = f32::from_le_bytes([
        data_vec[4100],
        data_vec[4101],
        data_vec[4102],
        data_vec[4103],
    ]);

    spe_data.frame = data_vec[4100..(4100 + frame_size * frame_counts * number_bytes)]
        .chunks(frame_size * number_bytes)
        .map(|frame| {
            if number_bytes == 2 {
                let one_frame = frame
                    .chunks(2)
                    .map(|piece| i16::from_le_bytes([piece[0], piece[1]]))
                    .collect::<Vec<i16>>();
                return FrameVec::I16FRAME(one_frame);
            } else if test_f32 < 1e-8 {
                let one_frame = frame
                    .chunks(4)
                    .map(|piece| i32::from_le_bytes([piece[0], piece[1], piece[2], piece[3]]))
                    .collect::<Vec<i32>>();
                return FrameVec::I32FRAME(one_frame);
            } else {
                let one_frame = frame
                    .chunks(4)
                    .map(|piece| f32::from_le_bytes([piece[0], piece[1], piece[2], piece[3]]))
                    .collect::<Vec<f32>>();
                return FrameVec::F32FRAME(one_frame);
            }
        })
        .collect::<Vec<FrameVec>>();

    spe_data.calc_maxs_mins();

    if data_vec.len() - (4100 + frame_size * frame_counts * number_bytes) > 0 {
        let string_try =
            String::from_utf8_lossy(&data_vec[4100 + frame_size * frame_counts * number_bytes..])
                .to_string();
        if string_try.find("<SpeFormat").is_none() {
            return serde_json::to_string(&spe_data).unwrap();
        }

        let mut reader = Reader::from_str(&string_try);

        let mut detector_angle: Vec<f64> = Vec::new();
        let mut focal_length: Vec<f64> = Vec::new();
        let mut inclusion_angle: Vec<f64> = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"Wavelength" => {
                        if e.try_get_attribute(QName(b"type")).unwrap().is_none() {
                            if let Event::Text(text) = reader.read_event().unwrap() {
                                let mut wavelength: Vec<f64> = Vec::new();
                                text.unescape()
                                    .unwrap()
                                    .split(',')
                                    .for_each(|val| wavelength.push(val.parse::<f64>().unwrap()));
                                spe_data.wavelength = Some(wavelength);
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
    }
    return serde_json::to_string(&spe_data).unwrap();
}

fn parse_txt(data_text: String) -> String {
    let mut spe_data = SpeData::init();

    let rows: Vec<&str> = data_text.trim().split_terminator('\n').collect();
    spe_data.width = rows.len();

    let (lambda_str, counts_str) = rows[0].trim()
        .split_once(&['\t', ',', ';', ' '][..])
        .unwrap();

    let mut wavelength = vec![lambda_str.parse::<f64>().unwrap()];

    let mut counts_rows = counts_str
        .split_terminator(&['\t', ',', ';', ' '][..])
        .map(|number_str| vec![number_str.parse::<i32>().unwrap()])
        .collect::<Vec<Vec<i32>>>();
    spe_data.height = counts_rows.len();

    rows[1..].iter().for_each(|row| {
        // let number_str_vec = str.split_terminator(&['\t', ',', ';', ' '][..]).collect::<Vec<&str>>();
        // number_str_vec.par_iter().enumerate().for_each(|(index, number_str)| {
        (*row).trim()
            .split_terminator(&['\t', ',', ';', ' '][..])
            .enumerate()
            .for_each(|(index, number_str)| {
                // let n = *number_str;
                if index == 0 {
                    wavelength.push(number_str.parse::<f64>().unwrap())
                } else {
                    counts_rows[index - 1].push(number_str.parse::<i32>().unwrap())
                }
            });
    });

    let counts_data = counts_rows.into_iter().flatten().collect::<Vec<i32>>();

    spe_data.wavelength = Some(wavelength);
    spe_data.frame = vec![FrameVec::I32FRAME(counts_data)];
    spe_data.calc_maxs_mins();

    return serde_json::to_string(&spe_data).unwrap();
}

#[tauri::command]
pub fn open_file(path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(path);
    match path_buf.extension().unwrap().to_str() {
        Some("spe") => {
            let data = fs::read(path_buf);
            if let Ok(data_vec) = data {
                Ok(parse_spe(data_vec))
            } else {
                Err(String::from("spe文件损坏"))
            }
        }
        Some(_) => {
            let data = fs::read_to_string(path_buf);
            if let Ok(data_text) = data {
                Ok(parse_txt(data_text))
            } else {
                Err(String::from("未知的文件格式"))
            }
        }
        None => Err(String::from("未知的文件格式")),
    }
}
