use std::{collections::HashMap, error::Error, fs::File, io::Read, path::Path};

use clap::Parser;


#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
  /// Input file
  file: String,
}

type StoreT = HashMap::<String, Vec<f64>>;

fn main() -> Result<(), Box<dyn Error>>{
  let args = Args::parse();
  let f_name = args.file;//.expect("You need to pass a file");
  let f_path = Path::new(&f_name);
  let mut buf = [0u8; 1024*512]; //Somethings fishy
  let mut f_handle = File::open(f_path)?;

  let mut storage = StoreT::new();

  let mut x = String::new();
  loop {
    match f_handle.read(&mut buf) {
      Ok(0) => break,
      Ok(_n) => {
        let v = Vec::from_iter(buf);
        (x, storage) = parse_foo(x + "\n" + &(String::from_utf8(v)?), storage)
      },
      // Err(e) => return Box::new(e),
      _ => panic!("TODO")
    }
  }
  // f_handle.read(&mut buf)?;
  // print!("{:#?}", &storage);

  dump_storage(storage);
  Ok(())
}

fn parse_foo(new_data: String, mut store: StoreT) -> (String, StoreT) {
  let mut lines = new_data.lines().peekable();
  let mut ret_data= String::new();
  while let Some(l) = lines.next() {
    if lines.peek().is_none() {
      ret_data = l.to_string();
      break;
    }
    if l.eq("") {
      continue;
    }
    let idx = l.find(';').expect(&format!("Couldn't find seperator (\";\") in \"{}\"", l));
    let (name, val) = l.split_at(idx);
    let val = val.chars().skip(1).collect::<String>();
    let name = name.to_string();
    let value = val.parse::<f64>().expect(&format!("Couldn't parse {} into float", val));
    if let Some(vals) = store.get_mut(&name) {
      vals.push(value)
    } else {
      store.insert(name, vec![value]);
    }
  }

  (ret_data, store)
}

fn dump_storage(storage: StoreT) {
  let mut names = Vec::from_iter(storage.keys());
  names.sort();
  println!("{{");
  let mut names = names.iter().peekable();
  // for &name in names {
  while let Some(&name) = names.next() {
    let vals = storage.get(name).expect(&format!("Couldn't retrieve values for {}", name));
    let mut min = 0.;
    let mut max = 0.;
    let mut avg = 0.;
    for &v in vals {
      if v < min {
        min = v
      } else if v > max {
        max = v
      }
      avg += v
    }
    avg/=vals.len() as f64;

    // Abha=5.0/18.0/27.4,
    if names.peek().is_none() {// is last -> no trailing comma
      println!("  {name}={min:.1}/{avg:.1}/{max:.1}")
    } else {
      println!("  {name}={min:.1}/{avg:.1}/{max:.1},")
    }
  }
  print!("}}");
}



