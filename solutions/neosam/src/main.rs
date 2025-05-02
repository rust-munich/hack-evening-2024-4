use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    rc::Rc,
};

const MAX_LOCATION_LENGTH: usize = 32;

pub fn get_input_filename() -> Option<String> {
    let args = env::args();
    if let Some(filename) = args.skip(1).next() {
        Some(filename)
    } else {
        None
    }
}

#[derive(Debug)]
pub struct Measurement<'a> {
    pub location: &'a [u8],
    pub temperature: f64,
}
impl<'a> Measurement<'a> {
    pub fn parse_from_bytes(buffer: &'a [u8]) -> Option<Measurement<'a>> {
        let start_location_index = 0;
        let end_location_index = (0..(buffer.len()))
            .map(|idx| (idx, buffer[idx]))
            .find(|(_idx, c)| *c == b';')?
            .0;
        let start_temperature_index = end_location_index + 1;
        let end_temparature_index = (start_location_index..(buffer.len()))
            .map(|idx| (idx, buffer[idx]))
            .find(|(_idx, c)| *c == b'\n')?
            .0;

        Some(Measurement {
            location: &buffer[start_location_index..end_location_index],
            temperature: parse_byte_array_to_f64(
                &buffer[start_temperature_index..end_temparature_index],
            ),
        })
    }
}

#[derive(Clone)]
pub struct StatisticData {
    pub location: Rc<[u8]>,
    pub min: f64,
    pub max: f64,
    pub average: f64,
    pub count: f64,
}
impl StatisticData {
    pub fn new(location: Rc<[u8]>, temperature: f64) -> Self {
        StatisticData {
            location,
            min: temperature,
            max: temperature,
            average: temperature,
            count: 1.0,
        }
    }

    pub fn add(&mut self, temperature: f64) {
        if self.min < temperature {
            self.min = temperature;
        }
        if self.max > temperature {
            self.max = temperature;
        }
        self.average = (self.average * self.count + temperature) / (self.count + 1.0);
        self.count += 1.0;
    }
}

pub fn parse_byte_array_to_f64(bytes: &[u8]) -> f64 {
    let mut buffer = 0.0;
    let mut state = 0;
    for b in bytes {
        if *b == b'-' {
            buffer *= -1.0;
        } else if *b >= b'0' && *b <= b'9' {
            if state == 0 {
                buffer = buffer * 10.0 + (b - b'0') as f64;
            } else {
                buffer += (b - b'0') as f64 / 10.0;
            }
        } else if *b == b'\n' {
            // Ignore the newline at the end
        } else if *b == b'.' {
            state = 1;
        } else {
            panic!(
                "Invalid temperature format: '{}' - {:?} - failed at {}",
                String::from_utf8_lossy(bytes),
                bytes,
                b
            );
        }
    }
    buffer
}

pub struct MeasurementParser<'a> {
    buffer: &'a [u8],
    index: usize,
}
impl<'a> Iterator for MeasurementParser<'a> {
    type Item = Measurement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start_location_index = self.index;
        let end_location_index = (self.index..(self.buffer.len()))
            .map(|idx| (idx, self.buffer[idx]))
            .find(|(_idx, c)| *c == b';')?
            .0;
        let start_temperature_index = end_location_index + 1;
        let end_temparature_index = (start_location_index..(self.buffer.len()))
            .map(|idx| (idx, self.buffer[idx]))
            .find(|(_idx, c)| *c == b'\n')?
            .0;
        self.index = end_temparature_index + 1;

        Some(Measurement {
            location: &self.buffer[start_location_index..end_location_index],
            temperature: parse_byte_array_to_f64(
                &self.buffer[start_temperature_index..end_temparature_index],
            ),
        })
    }
}

pub struct DataStore {
    data_map: heapless::FnvIndexMap<Rc<[u8]>, StatisticData, 16384>,
}
impl DataStore {
    pub fn new() -> Self {
        DataStore {
            data_map: heapless::FnvIndexMap::new(),
        }
    }

    pub fn add_entry<'a>(&mut self, measurement: Measurement<'a>) {
        let location: Rc<[u8]> = measurement.location.into();
        if !self.data_map.contains_key(&location) {
            self.data_map.insert(
                location.clone(),
                StatisticData::new(location, measurement.temperature),
            );
        } else {
            let statistic_data = self.data_map.get_mut(&location).unwrap();
            statistic_data.add(measurement.temperature);
        }
    }

    pub fn print(&self) {
        println!("{{");
        let count = self.data_map.len() - 1;
        let mut data: Vec<StatisticData> = self.data_map.values().cloned().collect();
        data.sort_by(|data1, data2| data1.location.cmp(&data2.location));
        for (statistic_data, i) in data.iter().zip(0usize..) {
            print!(
                "    {}={:.1}/{:.1}/{:.1}",
                String::from_utf8_lossy(statistic_data.location.as_ref()),
                statistic_data.min,
                statistic_data.max,
                statistic_data.average,
            );
            if i < count {
                println!(",");
            } else {
                println!();
            }
        }
        print!("}}");
    }
}

fn main() {
    let input_filename = get_input_filename().expect("Please specify the input file");
    let input_file = File::open(input_filename).expect("Could not open input file");
    let mut reader = BufReader::new(input_file);
    let mut data_store = DataStore::new();
    let mut overall_count = 0u32;
    loop {
        {
            //let buffer = reader.fill_buf().expect("Could not fill buffer");
            let mut buffer = Vec::with_capacity(1000);
            reader.read_until(b'\n', &mut buffer).unwrap();
            if buffer.len() == 0 {
                break;
            }
            //let mut measurement_iterator = MeasurementParser {
            //    buffer: buffer.as_ref(),
            //    index: 0,
            //};
            //for measurement in &mut measurement_iterator {
            let measurement =
                Measurement::parse_from_bytes(buffer.as_ref()).expect("Could not parse buffer");
            overall_count += 1;
            data_store.add_entry(measurement);
            //}
            //buffer.len()
        };
    }
    data_store.print();
}
