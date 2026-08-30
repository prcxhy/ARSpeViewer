//! 光谱数据文件解析：`.spe` 二进制格式与 `.txt`/`.csv`/`.asc` 文本格式。
//!
//! 解析逻辑自 src-tauri 原样平移（含解析分支与错误文案），仅将返回值从
//! JSON 字符串改为结构体，序列化交由各宿主（GUI / Python）自行处理。

use crate::error::CoreError;
use crate::model::SpeData;
use quick_xml::{escape::unescape, events::Event, reader::Reader};
use std::{fs, path::Path};

/// 解析 `.spe` 二进制数据。
///
/// 布局（参考 Princeton Instruments SPE 格式）：
/// - width @ 6 (u16 LE)、height @ 18 (u16 LE)、帧数 @ 1446 (u8)
/// - 像素数据 @ 4100 起，按 i16 / i32 / f32 自适应
/// - 数据之后若为 `<SpeFormat` XML 尾部，则读取波长与标定参数
pub fn parse_spe(data_vec: &[u8]) -> SpeData {
    let mut spe_data = SpeData::default();

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
                    .map(|piece| i16::from_le_bytes([piece[0], piece[1]]) as f64)
                    .collect::<Vec<f64>>();
                return one_frame;
            } else if test_f32 < 1e-8 {
                let one_frame = frame
                    .chunks(4)
                    .map(|piece| {
                        i32::from_le_bytes([piece[0], piece[1], piece[2], piece[3]]) as f64
                    })
                    .collect::<Vec<f64>>();
                return one_frame;
            } else {
                let one_frame = frame
                    .chunks(4)
                    .map(|piece| f32::from_le_bytes([piece[0], piece[1], piece[2], piece[3]]) as f64)
                    .collect::<Vec<f64>>();
                return one_frame;
            }
        })
        .collect::<Vec<Vec<f64>>>();

    spe_data.calc_maxs_mins();

    if data_vec.len() - (4100 + frame_size * frame_counts * number_bytes) > 0 {
        let string_try =
            String::from_utf8_lossy(&data_vec[4100 + frame_size * frame_counts * number_bytes..])
                .to_string();
        if string_try.find("<SpeFormat").is_none() {
            return spe_data;
        }

        let mut reader = Reader::from_str(&string_try);

        let mut detector_angle: Vec<f64> = Vec::new();
        let mut focal_length: Vec<f64> = Vec::new();
        let mut inclusion_angle: Vec<f64> = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    "Wavelength" => {
                        if e.try_get_attribute("type").unwrap().is_none() {
                            if let Event::Text(text) = reader.read_event().unwrap() {
                                let mut wavelength: Vec<f64> = Vec::new();
                                unescape(&text)
                                    .unwrap()
                                    .split(',')
                                    .for_each(|val| wavelength.push(val.parse::<f64>().unwrap()));
                                spe_data.wavelength = Some(wavelength);
                            }
                        }
                    }
                    "DetectorAngle" => {
                        if let Event::Text(text) = reader.read_event().unwrap() {
                            detector_angle.push(unescape(&text).unwrap().parse::<f64>().unwrap());
                        }
                    }
                    "FocalLength" => {
                        if let Event::Text(text) = reader.read_event().unwrap() {
                            focal_length.push(unescape(&text).unwrap().parse::<f64>().unwrap());
                        }
                    }
                    "InclusionAngle" => {
                        if let Event::Text(text) = reader.read_event().unwrap() {
                            inclusion_angle.push(unescape(&text).unwrap().parse::<f64>().unwrap());
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
    spe_data
}

/// 解析 `.txt`/`.csv`/`.asc` 文本数据。
///
/// 自动识别两种排布：
/// - 行主序：首行为波长行（波长 + 各列波长值），其后每行为一行数据；
/// - 列主序：数据行首列为波长，其后各列为各角度的计数值。
pub fn parse_txt(data_text: &str) -> Result<SpeData, CoreError> {
    let mut spe_data = SpeData::default();

    let rows: Vec<&str> = data_text.trim().split_terminator('\n').collect();

    let mut data_start_index: usize = 0;
    let mut row_major = true;
    let mut wavelength: Vec<f64> = Vec::new();
    let mut counts_rows: Vec<Vec<f64>> = Vec::new();

    for i in 0..rows.len() {
        if let Some((str_1, str_2)) = rows[i].trim().split_once(&['\t', ',', ';', ' '][..]) {
            if let Ok(lambda_start) = str_1.parse::<f64>() {
                let str_2_vec = str_2
                    .split_terminator(&['\t', ',', ';', ' '][..])
                    .collect::<Vec<&str>>();
                // 通过拆分长度判断是否是数据开始行
                if str_2_vec.len() < 100 {
                    continue;
                }
                wavelength.push(lambda_start);
                // 首行尾不是浮点数则说明波长在首列，数据按列存储
                if let Ok(_) = str_2_vec.last().unwrap().parse::<i32>() {
                    println!("wavelength is the first column");
                    row_major = false;
                    spe_data.height = str_2_vec.len();
                    spe_data.width = 1;
                    str_2_vec.into_iter().for_each(|number_str| {
                        counts_rows.push(vec![number_str.parse::<f64>().unwrap()])
                    });
                } else {
                    println!("wavelength is the first row");
                    spe_data.width = str_2_vec.len() + 1;
                    str_2_vec.into_iter().for_each(|str| {
                        wavelength.push(str.parse::<f64>().unwrap());
                    });
                }
                data_start_index = i + 1;
                break;
            }
        }
    }

    if data_start_index == 0 {
        return Err(CoreError::NoValidData);
    }

    let mut frame_data: Vec<f64> = Vec::new();

    if row_major {
        for i in data_start_index..rows.len() {
            let number_strings = rows[i]
                .trim()
                .split_terminator(&['\t', ',', ';', ' '][..])
                .collect::<Vec<&str>>();
            if number_strings.len() == spe_data.width {
                number_strings.into_iter().for_each(|str| {
                    frame_data.push(str.parse::<f64>().unwrap());
                });
                spe_data.height += 1;
            } else {
                break;
            }
        }
    } else {
        for i in data_start_index..rows.len() {
            let number_strings = rows[i]
                .trim()
                .split_terminator(&['\t', ',', ';', ' '][..])
                .collect::<Vec<&str>>();
            if number_strings.len() == spe_data.height + 1 {
                number_strings
                    .into_iter()
                    .enumerate()
                    .for_each(|(index, number_str)| {
                        if index == 0 {
                            wavelength.push(number_str.parse::<f64>().unwrap())
                        } else {
                            counts_rows[index - 1].push(number_str.parse::<f64>().unwrap())
                        }
                    });
                spe_data.width += 1;
            } else {
                break;
            }
        }
        frame_data = counts_rows.into_iter().flatten().collect::<Vec<f64>>();
    }

    spe_data.wavelength = Some(wavelength);
    spe_data.frame = vec![frame_data];
    spe_data.calc_maxs_mins();

    Ok(spe_data)
}

/// 读取文件并按扩展名分流解析（对应 GUI 的 `open_file` 命令）。
pub fn open_path(path: &Path) -> Result<SpeData, CoreError> {
    let extention = if let Some(ext) = path.extension() {
        ext.to_str()
    } else {
        None
    };
    match extention {
        Some("spe") => {
            let data = fs::read(path);
            if let Ok(data_vec) = data {
                Ok(parse_spe(&data_vec))
            } else {
                Err(CoreError::SpeFileCorrupted)
            }
        }
        Some(_) => {
            let data = fs::read_to_string(path);
            if let Ok(data_text) = data {
                parse_txt(&data_text)
            } else {
                Err(CoreError::UnknownFileFormat)
            }
        }
        None => Err(CoreError::FolderNotSupported),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 生成最小 .spe 二进制（i32 数据路径，无 XML 尾部）。
    fn make_spe_i32(width: u16, height: u16, counts: u8, samples: &[i32], xml_tail: Option<&str>) -> Vec<u8> {
        let frame_size = width as usize * height as usize * counts as usize;
        let mut data = vec![0u8; 4100 + frame_size * 4];
        data[6..8].copy_from_slice(&width.to_le_bytes());
        data[18..20].copy_from_slice(&height.to_le_bytes());
        data[1446] = counts;
        for (i, v) in samples.iter().enumerate() {
            data[4100 + i * 4..4100 + i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        if let Some(xml) = xml_tail {
            data.extend_from_slice(xml.as_bytes());
        }
        data
    }

    #[test]
    fn parse_spe_i32_and_min_max() {
        let data = make_spe_i32(2, 3, 1, &[1, 2, 3, 4, 5, 6], None);
        let spe = parse_spe(&data);
        assert_eq!(spe.width, 2);
        assert_eq!(spe.height, 3);
        assert_eq!(spe.frame, vec![vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]]);
        assert_eq!(spe.min_max, vec![vec![1.0, 6.0]]);
        assert!(spe.wavelength.is_none());
        assert_eq!(spe.detector_angle_cal, 0.0);
    }

    #[test]
    fn parse_spe_f32_data() {
        let frame: Vec<f32> = vec![10.0, 20.0, 30.0];
        let mut data = vec![0u8; 4100 + frame.len() * 4];
        data[6..8].copy_from_slice(&1u16.to_le_bytes());
        data[18..20].copy_from_slice(&3u16.to_le_bytes());
        data[1446] = 1;
        for (i, v) in frame.iter().enumerate() {
            data[4100 + i * 4..4100 + i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        // 首个样本 >= 1e-8 → f32 路径
        let spe = parse_spe(&data);
        assert_eq!(spe.frame, vec![vec![10.0, 20.0, 30.0]]);
        assert_eq!(spe.min_max, vec![vec![10.0, 30.0]]);
    }

    #[test]
    fn parse_spe_i16_data() {
        // i16 路径触发条件：frame_size * counts * 4 > 文件总长，
        // 需要 > 2050 个样本（width=100, height=25 → 2500）。
        let width: u16 = 100;
        let height: u16 = 25;
        let samples: Vec<i16> = (0..2500).map(|i| i as i16).collect();
        let mut data = vec![0u8; 4100 + samples.len() * 2];
        data[6..8].copy_from_slice(&width.to_le_bytes());
        data[18..20].copy_from_slice(&height.to_le_bytes());
        data[1446] = 1;
        for (i, v) in samples.iter().enumerate() {
            data[4100 + i * 2..4100 + i * 2 + 2].copy_from_slice(&v.to_le_bytes());
        }
        let spe = parse_spe(&data);
        assert_eq!(spe.width, 100);
        assert_eq!(spe.height, 25);
        assert_eq!(spe.frame.len(), 1);
        assert_eq!(spe.frame[0][0], 0.0);
        assert_eq!(spe.frame[0][2499], 2499.0);
    }

    #[test]
    fn parse_spe_with_xml_calibration() {
        let xml = "<SpeFormat>\
<Wavelength>400.0,500.0</Wavelength>\
<DetectorAngle>10.5</DetectorAngle><FocalLength>300.0</FocalLength><InclusionAngle>20.0</InclusionAngle>\
<DetectorAngle>30.5</DetectorAngle><FocalLength>400.0</FocalLength><InclusionAngle>40.0</InclusionAngle>\
</SpeFormat>";
        let data = make_spe_i32(1, 2, 1, &[7, 8], Some(xml));
        let spe = parse_spe(&data);
        assert_eq!(spe.wavelength, Some(vec![400.0, 500.0]));
        assert_eq!(spe.detector_angle_cal, 10.5);
        assert_eq!(spe.focal_length_cal, 300.0);
        assert_eq!(spe.inclusion_angle_cal, 20.0);
        assert_eq!(spe.detector_angle_exp, 30.5);
        assert_eq!(spe.focal_length_exp, 400.0);
        assert_eq!(spe.inclusion_angle_exp, 40.0);
    }

    #[test]
    fn parse_spe_non_xml_tail_ignored() {
        let data = make_spe_i32(1, 1, 1, &[5], Some("plain text tail"));
        let spe = parse_spe(&data);
        assert!(spe.wavelength.is_none());
    }

    /// 行主序文本：首行波长（列数需 >= 100 才被识别为数据起始行）。
    fn make_row_major_txt(data_rows: usize, cols: usize) -> String {
        let mut lines: Vec<String> = Vec::new();
        let wl: Vec<String> = (0..cols).map(|i| format!("{:.1}", 400.0 + i as f64)).collect();
        lines.push(wl.join("\t"));
        for r in 1..=data_rows {
            let row: Vec<String> = (0..cols).map(|i| format!("{:.1}", r as f64 * 1000.0 + i as f64)).collect();
            lines.push(row.join("\t"));
        }
        lines.join("\n")
    }

    #[test]
    fn parse_txt_row_major() {
        let spe = parse_txt(&make_row_major_txt(3, 101)).unwrap();
        assert_eq!(spe.width, 101);
        assert_eq!(spe.height, 3);
        assert_eq!(spe.wavelength.as_ref().unwrap().len(), 101);
        assert_eq!(spe.wavelength.as_ref().unwrap()[0], 400.0);
        assert_eq!(spe.wavelength.as_ref().unwrap()[100], 500.0);
        // 第一数据行
        assert_eq!(spe.frame[0][0], 1000.0);
        assert_eq!(spe.frame[0][100], 1100.0);
        // 第二数据行（行存于 frame[0] 内部，步长为 width）
        assert_eq!(spe.frame[0][2 * 101], 3000.0);
        assert_eq!(spe.min_max, vec![vec![1000.0, 3100.0]]);
    }

    #[test]
    fn parse_txt_column_major() {
        // 列主序：首数据行 = 波长 + 各角度计数（计数值为整数格式触发该分支）
        let mut lines: Vec<String> = Vec::new();
        // 首数据行：波长 400 + 100 个角度的计数
        lines.push(format!("400.0\t{}", (0..100).map(|a| (100 + a).to_string()).collect::<Vec<_>>().join("\t")));
        for l in 1..5 {
            lines.push(format!(
                "{:.1}\t{}",
                400.0 + l as f64,
                (0..100).map(|a| (200 + l * 10 + a).to_string()).collect::<Vec<_>>().join("\t")
            ));
        }
        let spe = parse_txt(&lines.join("\n")).unwrap();
        assert_eq!(spe.height, 100);
        assert_eq!(spe.width, 5);
        assert_eq!(spe.wavelength.as_ref().unwrap().len(), 5);
        assert_eq!(spe.wavelength.as_ref().unwrap()[0], 400.0);
        // 角度 0 的数据行：100, 210, 220, 230, 240
        assert_eq!(spe.frame[0][0], 100.0);
        assert_eq!(spe.frame[0][1], 210.0);
        assert_eq!(spe.frame[0][4], 240.0);
    }

    #[test]
    fn parse_txt_invalid() {
        assert_eq!(parse_txt("hello world"), Err(CoreError::NoValidData));
    }

    #[test]
    fn serde_round_trip() {
        let spe = parse_txt(&make_row_major_txt(2, 101)).unwrap();
        let json = serde_json::to_string(&spe).unwrap();
        let back: SpeData = serde_json::from_str(&json).unwrap();
        assert_eq!(spe, back);
    }
}
