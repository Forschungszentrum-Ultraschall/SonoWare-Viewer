use std::fs::File;
use std::vec;
use regex::Regex;
use ndarray::{Array, ArrayBase, OwnedRepr, Dim, s};
use serde::{Serialize, Deserialize};
use iir_filters::sos::zpk2sos;
use iir_filters::filter::{DirectForm2Transposed, Filter};
use iir_filters::filter_design::{butter, FilterType};

#[derive(Serialize, Deserialize)]
struct FilterConfig {
    order: u32,
    min_freq: f64,
    max_freq: f64
}

/// The header of a loaded dataset
#[derive(Default, Serialize, Clone)]
pub struct Header {
    /// Data format used for recording the data
    format: String,
    /// Version of the data format
    version: String,
    /// Number of axes
    axes: u8,
    /// Distance between two points in horizontal direction
    pub res_x: f32,
    /// Distance between two points in vertical direction
    pub res_y: f32,
    /// Number of samples in horizontal direction
    pub samples_x: u16,
    /// Number of samples in vertical direction
    pub samples_y: u16,
    /// List containing information about each data subset
    sub_sets: Vec<SubSet>,
    /// Number of recorded channels
    channels: u8,
    /// Number of samples per A-Scan
    samples: u32
}

/// Information about a subset of the loaded data
#[derive(Serialize, Clone)]
pub struct SubSet {
    /// Subset name
    name: String,
    /// Size of one sample in bytes
    element_size: u8,
    /// Number of samples in this subset
    sample_nums: u32,
    /// Minimum sample value
    pub min_sample_pos: f32,
    /// Resolution of the samples
    pub sample_resolution: f32,
    /// Gain for the given subset
    pub gain: f64
}

/// Structure for loaded ultrasonic data
#[derive(Default)]
pub struct UsData {
    /// data header
    pub header: Header,
    /// Recorded channels with their data
    datasets: Vec<ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>>
}

impl UsData {
    /// Loads the data recorded with SonoWare
    /// 
    /// # Arguments
    /// * `data`: binary content of the data file
    /// 
    /// # Returns
    /// If the data can be loaded successfully, an `UsData` struct
    /// is returned, else **None**
    pub fn load_sonoware(data: Vec<u8>) -> Option<UsData> {
        parse_sonoware_file(data)
    }

    /// Returns the data of a specific channel
    /// 
    /// # Arguments
    /// 
    /// * `channel`: requested channel number
    /// 
    /// # Returns
    /// If the channel has been recorded the array storing its
    /// values will be returned, else **None**
    pub fn get_channel(&self, channel: usize) -> Option<&ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>> {
        if &self.datasets.len() > &channel {
            Some(&self.datasets[channel])
        }
        else {
            None
        }
    }

    /// Get the subset settings for a specific channel
    /// 
    /// # Arguments
    /// * `channel`: Channel number
    /// 
    /// # Returns
    /// If the channel has been recorded, its subset setting
    /// will be returned, else **None**
    pub fn get_channel_subset(&self, channel: usize) -> Option<&SubSet> {
        let mut i = 0;

        for subset in &self.header.sub_sets {
            if subset.name.contains("Data") {
                if i == channel {
                    return Some(subset);
                }
                i += 1;
            }
        }

        None
    }

    /// Generates the C-Scan of a specific channel
    /// # Arguments
    /// * `channel`: Channel number
    /// * `start`: Start index for the aperture
    /// * `end`: End index for the aperture
    /// * `as_decibel`: Maximum should be returned as dB value
    /// 
    /// # Returns
    /// If the channel has been recorded a 2-D array containing the maximum of
    /// each data point will be returned, else **None**
    pub fn c_scan(&self, channel: usize, start: usize, end: usize, as_decibel: bool) -> Option<ArrayBase<OwnedRepr<f64>, Dim<[usize; 2]>>> {
        let data = self.get_channel(channel);
        let gain = self.get_channel_subset(channel).unwrap().gain;

        match data {
            Some(array) => {
                let shape = array.shape();

                let mut scan: ArrayBase<OwnedRepr<f64>, Dim<[usize; 2]>> = Array::zeros((shape[0], shape[1]));

                for (row_index, row) in array.outer_iter().enumerate() {
                    for (col_index, col) in row.outer_iter().enumerate() {
                        let window = col.slice(s![start..end]);
                        let filtered_window = filter_a_scan(&window.to_vec()).unwrap();

                        let mut maximum: f64 = filtered_window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

                        if as_decibel {
                            maximum = 20.0 * (maximum * (f64::powi(2.0, 15) - 1.0)).abs().log10() - gain;
                        }
                        
                        scan[[row_index, col_index]] = maximum;
                    }
                }

                Some(scan)
            }

            None => {
                println!("Invalid channel request");
                None
            }
        }
    }

    /// Generates the D-Scan of a requested channel
    /// 
    /// # Arguments
    /// * `channel`: Channel number
    /// * `start`: Start index of the aperture
    /// * `end`: End index of the aperture
    /// 
    /// # Returns
    /// If the channel has been recorded a 2-D array containing the Argmax
    /// inside the aperture of each datapoint will be returned, else **None**
    pub fn d_scan(&self, channel: usize, start: usize, end: usize) -> Option<ArrayBase<OwnedRepr<u32>, Dim<[usize; 2]>>> {
        let data_link = self.get_channel(channel);

        match data_link {
            Some(data) => {
                let shape = data.shape();

                let mut scan = Array::zeros((shape[0], shape[1]));

                for (row_index, row) in data.outer_iter().enumerate() {
                    for (col_index, col) in row.outer_iter().enumerate() {
                        let window = col.slice(s![start..end]);
                        let filtered_window = filter_a_scan(&window.to_vec()).unwrap();

                        let maximum = filtered_window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                        let argmax = filtered_window.iter().position(|x| x == &maximum).unwrap();

                        scan[[row_index, col_index]] = argmax as u32 + start as u32;
                    }
                }

                Some(scan)
            }
            None => {
                println!("Invalid channel request");
                None
            }
        }
    }
}

/// Parse the binary content of a SonoWare file
/// 
/// # Arguments
/// * `binary_data`: Byte-Array containing SonoWare file content
/// 
/// # Returns
/// If the file can be parsed without issues a `UsData` struct
/// containing the data will be returned, else **None** 
fn parse_sonoware_file(binary_data: Vec<u8>) -> Option<UsData> {
    let string_data = String::from_utf8_lossy(binary_data.as_slice());
    let header_ending = "|^Data Set^|";

    let header_end = string_data.find(header_ending);

    let mut gains: Vec<f64> = vec![0.0];

    gains.extend(Regex::new("\"Gain\">\\d+").unwrap().find_iter(&string_data)
        .filter_map(|number| number.as_str()[7..].parse::<f64>().ok()));

    match header_end {
        Some(index) => {
            let header_string = String::from_utf8(binary_data[..index].to_vec()).unwrap();
            let header = parse_header(header_string, gains);

            let mut us_data = UsData {
                header,
                datasets: vec![]
            };

            let samples_x = &us_data.header.samples_x;
            let samples_y = &us_data.header.samples_y;
            let subsets = &us_data.header.sub_sets;

            let mut data_bytes = binary_data[index + header_ending.len() + 3..].iter().collect::<Vec<_>>();

            let points = *samples_x as u32 * *samples_y as u32;
            for subset in subsets {
                let values = subset.element_size as u32 * subset.sample_nums * points;
                let sub_sample = data_bytes[..values as usize].to_vec();
                data_bytes.drain(0..values as usize);

                if subset.name.contains("Data") {
                    let sub_data = get_raw_data(&sub_sample, &subset, *samples_x, *samples_y);

                    us_data.datasets.push(sub_data);
                }
            }

            Some(us_data)
        }
        None => {
            println!("Failed to load header");
            None
        }
    }
}

/// Parsing the header string of a data file
/// 
/// # Arguments
/// * `header`: String representation of the header
/// 
/// # Returns
/// A `Header` struct containing the data of the provided header
fn parse_header(header: String, gains: Vec<f64>) -> Header {
    let lines = header.lines().collect::<Vec<_>>();
    let format = get_entry(lines[0]);
    let version = get_entry(lines[1]);
    let axes = get_entry(lines[3]).parse::<u8>().unwrap();
    let subsets = get_entry(lines[4]).parse::<u8>().unwrap();
    let res_x = get_float_entry(lines[8]).unwrap();
    let res_y = get_float_entry(lines[12]).unwrap();
    let samples_x = parse_entry::<u16>(lines[6]).unwrap();
    let samples_y = parse_entry::<u16>(lines[10]).unwrap();

    let mut sub_sets = vec![];

    let mut samples = 0;

    for i in 0..subsets {
        let skip = i * 12;

        sub_sets.push(SubSet { 
            name: get_entry(lines[14 + skip as usize]), 
            element_size: parse_entry::<u8>(lines[15 + skip as usize]).unwrap(), 
            sample_nums: parse_entry::<u32>(lines[17 + skip as usize]).unwrap(),
            min_sample_pos: get_float_entry(lines[18 + skip as usize]).unwrap(),
            sample_resolution: get_float_entry(lines[19 + skip as usize]).unwrap(),
            gain: gains[i as usize]
        });

        if sub_sets.last().unwrap().sample_nums > samples {
            samples = sub_sets.last().unwrap().sample_nums;
        }
    }

    let channels = sub_sets.iter().filter(|&n| (*n).name.contains("Data")).count() as u8;

    Header { 
        format, 
        version, 
        axes,
        res_x, 
        res_y, 
        samples_x, 
        samples_y, 
        sub_sets, 
        channels,
        samples 
    }
}

/// Parses a header line to get the value
/// 
/// # Arguments
/// * `line`: Single line of the header
/// 
/// # Returns
/// An empty `String` if no separated value is available else
/// a `String` containing the provided value
fn get_entry(line: &str) -> String {
    match String::from(line).split(": ").last() {
        Some(value) => { String::from(value) }
        None => { String::from("") }
    }
}

/// Reads a line and converts it into a `T` object
/// 
/// # Arguments
/// * `line`: Single line of the header file
/// 
/// # Returns
/// If the value of the line can be convert into `T` the
/// converted value will be returned else **None**
fn parse_entry<T>(line: &str) -> Option<T> where T: std::str::FromStr {
    let value_string = get_entry(line);

    match value_string.parse::<T>() {
        Ok(value) => { Some(value) }
        Err(_) => {
            println!("Error while parsing: {}", value_string);
            None
        }
    }
}

/// Reads a header line and converts the value into a float value
/// 
/// # Arguments
/// * `line`: Single line of the header
/// 
/// # Returns
/// If the value can be converted into a `f32` the converted value
/// will be returned else **None**
fn get_float_entry(line: &str) -> Option<f32> {
    let string_value = get_entry(line);

    match string_value[..string_value.len() - 3].parse::<f32>() {
        Ok(value) => { Some(value) }
        Err(error) => {
            println!("Error: {}", error);
            None
        }
    }
}

/// Reads the binary data of a channel and returns a 3-D-Array containing
/// all recorded A-Scans
/// 
/// # Arguments
/// * `data`: raw data of the channel
/// * `sub_set`: `SubSet` containing the configuration of the channel
/// * `x`: number of columns in the dataset
/// * `y`: number of rows in the dataset
/// 
/// # Returns
/// A 3-D-Array of shape `[y, x, SubSet.samples]` is returned containing the values
/// as `i16`.
fn get_raw_data(data: &Vec<&u8>, sub_set: &SubSet, x: u16, y: u16) -> ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>> {    
    let mut array: ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>> = Array::zeros((y as usize, x as usize, sub_set.sample_nums as usize));
    
    let mut i = 0;

    for chunk in data.chunks(sub_set.element_size as usize) {
        let mut bytes: [u8; 2] = [0, 0];

        bytes[0] = *chunk[0];
        bytes[1] = *chunk[1];

        let sample = i % sub_set.sample_nums;
        let col = (i / sub_set.sample_nums) % x as u32;
        let row = i / (sub_set.sample_nums * x as u32);

        let value = (i16::from_be_bytes(bytes) as f32 - i16::MIN as f32) / (i16::MAX as f32 - i16::MIN as f32) * 2.0 - 1.0;

        array[[row as usize, col as usize, sample as usize]] = value;
        i += 1;
    }

    array
}

pub fn filter_a_scan(a_scan: &Vec<f32>) -> Option<Vec<f64>> {
    let mut output = vec![];

    let config: FilterConfig = serde_json::from_reader(File::open("filter_config.json").unwrap()).unwrap();

    let fs = 1e4;
    let zpk = butter(config.order, FilterType::BandPass(config.min_freq, config.max_freq), fs).unwrap();

    let sos = zpk2sos(&zpk, None).unwrap();
    let mut filtering = DirectForm2Transposed::new(&sos);

    for sample in a_scan.iter() {
        output.push(filtering.filter((*sample).into()));
    }
    
    Some(output)
}
