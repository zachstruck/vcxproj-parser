#[macro_use]
extern crate clap;
extern crate encoding_rs;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate regex;
extern crate sxd_document;
mod condition;
mod vcxproj;
use clap::App;
use vcxproj::Vcxproj;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let vcxproj_filename = matches.value_of("VCXPROJ").unwrap();

    let mut data = Vcxproj::new();
    if data.read_vcxproj(vcxproj_filename).is_err() {
        // FIXME  Get a better error message
        eprintln!("Error parsing");
        return;
    }

    for (key, value) in &data.values {
        println!("Key: {} | Value: {}", key, value);
    }
}
