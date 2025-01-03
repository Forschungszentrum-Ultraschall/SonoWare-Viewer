#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Read;
    use ndarray::s;

    use crate::data::UsData;

    const DATA_DIR: &str = "test_scans";

    #[test]
    fn start_scan() {
         run_test_on("test_scans/AScanDummy_0_0.itx", 0, 0);
    }

    #[test]
    fn mid_scan() {
        run_test_on("test_scans/AScanDummy_17_42.itx", 17, 42);
    }

    #[test]
    fn end_scan() {
        run_test_on("test_scans/AScanDummy_91_56.itx", 91, 56);
    }

    fn run_test_on(ref_path: &str, x: usize, y: usize) {
        let file = fs::read_to_string(ref_path).unwrap();

        let ref_scan: Vec<i16> = file.lines().map(|line| line.trim().parse::<i16>().unwrap()).collect();
        
        for file in fs::read_dir(DATA_DIR).unwrap() {
            match file {
                Ok(file_path) => {
                    let name = file_path.file_name();

                    if name.into_string().unwrap().contains("Stein 1-02.sdt") {
                        let mut file_content = File::open(file_path.path()).expect("File not found!");
                        let mut data_vec: Vec<u8> = Vec::new();

                        file_content.read_to_end(&mut data_vec).expect("Failed to read file!");

                        let data = UsData::load_sonoware(data_vec);

                        match data {
                            Some(dataset) => check_scan(dataset, &ref_scan, x, y),
                            None => panic!("Failed to load data")
                        }
                    }
                }
                Err(error) => {
                    dbg!("{}", error);
                    panic!("OS operation failed");
                }
            }
        }
    }

    fn check_scan(calc: UsData, reference: &Vec<i16>, x: usize, y: usize) {
        let a_scan = calc.get_channel(0).unwrap();
        let start = a_scan.slice(s![x, y, ..]);

        assert_eq!(start.len(), reference.len(), "Arrays need to have same length");
        
        let mut error_pos: Vec<usize> = vec![];

        for i in 0..reference.len() {
            let reference_value = (*reference.get(i).unwrap() as f64 - i16::MIN as f64) / (i16::MAX as f64 - i16::MIN as f64) * 2.0 - 1.0;
            let calc_value: f64 = start[i];

            if calc_value != reference_value {
                error_pos.push(i);
            }
        }

        assert_eq!(error_pos.len(), 0, "There should be no wrong values, but they dismatch at {:?}", error_pos);
    }
}