use std::fs::{self, DirEntry};

struct Header {
    format: String,
    version: String,
    axes: u8,
    subsets: u8,
    res_x: f32,
    res_y: f32,
    samples_x: u16,
    samples_y: u16,
    sub_sets: Vec<SubSet>,
    channels: u8
}

struct SubSet {
    name: String,
    element_size: u8,
    sample_nums: u32
}

pub(crate) fn load_sonoware() {
    let path = "/home/oliver/Viewer_LuftUS/Daten/2022-06-15 MgCr Steine Radenthein/Serie 1";

    for entry in fs::read_dir(path).unwrap() {
        match entry {
            Ok(file) => {
                let name = file.file_name();

                if name.into_string().unwrap().ends_with(".sdt") {
                    parse_file(file);
                }
            },
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }
}

fn parse_file(file: DirEntry) {
    let data = fs::read(file.path());

    match data {
        Ok(binary_data) => {
            let string_data = String::from_utf8_lossy(binary_data.as_slice());
            let header_ending = "|^Data Set^|";

            let header_end = string_data.find(header_ending);

            match header_end {
                Some(index) => {
                    let header_string = String::from_utf8(binary_data[..index].to_vec()).unwrap();
                    let header = parse_header(header_string);

                    let mut skip = 0;

                    for subset in header.sub_sets {
                        skip += subset.element_size as u32 * subset.sample_nums;
                    }

                    let data_bytes = binary_data[index + header_ending.len() + skip as usize..].iter().collect::<Vec<_>>();

                    println!("{}", data_bytes.len());
                }
                None => {}
            }
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }
}

fn parse_header(header: String) -> Header {
    let lines = header.lines().collect::<Vec<_>>();
    let format = get_entry(lines[0]);
    let version = get_entry(lines[1]);
    let axes = get_entry(lines[3]).parse::<u8>().unwrap();
    let subsets = get_entry(lines[4]).parse::<u8>().unwrap();
    let res_x = get_float_entry(lines[8]);
    let res_y = get_float_entry(lines[12]);
    let samples_x = parse_entry::<u16>(lines[6]);
    let samples_y = parse_entry::<u16>(lines[10]);

    let mut subsets_config = Vec::<SubSet>::new();

    for i in 0..subsets {
        let skip = i * 12;

        subsets_config.push(SubSet { 
            name: get_entry(lines[14 + skip as usize]), 
            element_size: parse_entry::<u8>(lines[15 + skip as usize]), 
            sample_nums: parse_entry::<u32>(lines[17 + skip as usize])
        });
    }

    let channels = subsets_config.iter().filter(|&n| (*n).name.contains("Data")).count() as u8;

    Header { format, version, axes, subsets, res_x, res_y, samples_x, samples_y, sub_sets: subsets_config, channels: channels }
}

fn get_entry(line: &str) -> String {
    match String::from(line).split(": ").last() {
        Some(value) => { String::from(value) }
        None => { String::from("") }
    }
}

fn parse_entry<T>(line: &str) -> T where T: std::str::FromStr {
    let value_string = get_entry(line);

    match value_string.parse::<T>() {
        Ok(value) => { value }
        Err(_) => {
            println!("Error while parsing: {}", value_string);
            panic!("Parsing header failed");
        }
    }
}

fn get_float_entry(line: &str) -> f32 {
    let string_value = get_entry(line);

    match string_value[..string_value.len() - 3].parse::<f32>() {
        Ok(value) => { value }
        Err(error) => {
            println!("Error: {}", error);
            panic!("Parsing header failed!");
        }
    }
}
