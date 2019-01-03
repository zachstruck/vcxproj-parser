#[macro_use]
extern crate clap;
use clap::App;
use std::fs;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let vcxproj_filename = matches.value_of("VCXPROJ").unwrap();

    let vcxproj = fs::read_to_string(vcxproj_filename).unwrap();

    println!("{}", vcxproj);
}
