#[macro_use]
extern crate clap;
extern crate sxd_document;
use clap::App;
use std::fs;
use sxd_document::dom;
use sxd_document::parser;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let vcxproj_filename = matches.value_of("VCXPROJ").unwrap();

    let vcxproj = fs::read_to_string(vcxproj_filename).unwrap();

    let package = parser::parse(&vcxproj).unwrap();
    let document = package.as_document();

    let root = document.root();
    for child in root.children() {
        match child.element() {
            Some(elem) => traverse(&elem),
            None => (),
        };
    }
}

fn traverse(elem: &dom::Element) {
    println!("{:?}", elem.name());
    for child in elem.children() {
        match child.element() {
            Some(elem) => {
                traverse(&elem);
            }
            None => (),
        }
    }
}
