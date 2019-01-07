#[macro_use]
extern crate clap;
extern crate sxd_document;
use clap::App;
use std::collections::HashMap;
use std::fs;
use sxd_document::dom;
use sxd_document::dom::ChildOfElement;
use sxd_document::dom::ChildOfRoot;
use sxd_document::parser;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let vcxproj_filename = matches.value_of("VCXPROJ").unwrap();

    let vcxproj = fs::read_to_string(vcxproj_filename).unwrap();

    let package = parser::parse(&vcxproj).unwrap();
    let document = package.as_document();

    let mut data = Data::new();

    let root = document.root();
    for child in root.children() {
        match child {
            ChildOfRoot::Element(elem) => data.traverse_element(&elem),
            ChildOfRoot::Comment(comment) => data.traverse_comment(&comment),
            ChildOfRoot::ProcessingInstruction(pi) => data.traverse_processing_instruction(&pi),
        };
    }

    for (key, value) in &data.values {
        println!("Key: {} | Value: {}", key, value);
    }
}

struct Data {
    values: HashMap<String, String>,
}

impl Data {
    fn new() -> Data {
        Data {
            values: HashMap::new(),
        }
    }

    fn traverse_element(&mut self, node: &dom::Element) {
        for child in node.children() {
            match child {
                ChildOfElement::Element(node) => self.traverse_element(&node),
                ChildOfElement::Text(text) => {
                    let val = self.traverse_text(&text);
                    if !val.is_empty() {
                        let key = node.name().local_part();
                        match self.values.insert(key.to_string(), val.to_string()) {
                            Some(old) => {
                                println!("Key: {} | Replacing \"{}\" with \"{}\"", key, old, val)
                            }
                            None => (),
                        }
                    }
                }
                ChildOfElement::Comment(comment) => self.traverse_comment(&comment),
                ChildOfElement::ProcessingInstruction(pi) => {
                    self.traverse_processing_instruction(&pi)
                }
            };
        }
    }

    fn traverse_text<'a>(&self, text: &'a dom::Text) -> &'a str {
        text.text().trim()
    }

    fn traverse_comment(&self, _comment: &dom::Comment) {}

    fn traverse_processing_instruction(&self, _pi: &dom::ProcessingInstruction) {}
}
