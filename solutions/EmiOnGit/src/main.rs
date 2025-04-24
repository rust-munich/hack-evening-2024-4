use std::{collections::HashMap, env, io::BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = args
        .get(1)
        .cloned()
        .unwrap_or("../../samples/weather_100.csv".into());
    let Ok(input_file) = std::fs::File::open(file_path.clone()) else {
        panic!("Could not read {file_path}");
    };
    let mut reader = std::io::BufReader::new(input_file);
    let mut line = String::new();
    let mut map: HashMap<String, Node> = HashMap::new();
    while let Ok(bytes_read) = reader.read_line(&mut line) {
        // EOF reached
        if bytes_read == 0 {
            break;
        }
        let Some((id, temperatur)) = line.split_once(";") else {
            line.clear();
            continue;
        };
        // whitespaces might make the parsing fail so we split it off
        let Ok(temperatur) = temperatur.split_whitespace().next().unwrap().parse() else {
            line.clear();
            continue;
        };
        if let Some(v) = map.get_mut(id) {
            v.push(temperatur);
        } else {
            map.insert(id.to_string(), Node::new(id.to_string(), temperatur));
        }
        line.clear();
    }
    let mut expected: Vec<String> = map
        .into_iter()
        .map(|(id, value)| {
            format!(
                "{id}={:.1}/{:.1}/{:.1},",
                value.min,
                // TODO do we round up?
                value.mean,
                value.max
            )
        })
        .collect();
    expected.sort();
    let expected_str = expected.join("\n");
    let result = "{\n".to_string() + &expected_str + "\n}";
    println!("{result}");
}
struct Node {
    // pub id: String,
    pub max: f64,
    pub min: f64,
    pub mean: f64,
    pub count: usize,
}
impl Node {
    pub fn new(id: String, first_temp: f64) -> Node {
        Node {
            // id,
            max: first_temp,
            min: first_temp,
            mean: first_temp,
            count: 1,
        }
    }
    pub fn push(&mut self, temp: f64) {
        self.max = self.max.max(temp);
        self.min = self.min.min(temp);
        self.count = self.count + 1;
        let p = 1. / self.count as f64;
        self.mean = self.mean * (1. - p) + temp * p;
    }
}
