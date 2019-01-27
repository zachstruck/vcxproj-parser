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
    config: String,
}

impl Data {
    fn new() -> Data {
        Data {
            values: HashMap::new(),
            config: String::new(),
        }
    }

    fn update_map(&mut self, elem: &dom::Element, text: &dom::Text) {
        let val = self.traverse_text(&text);
        if !val.is_empty() {
            let key = elem.name().local_part();
            match self.values.insert(key.to_string(), val.to_string()) {
                Some(old) => println!("Key: {} | Replacing \"{}\" with \"{}\"", key, old, val),
                None => (),
            }
        }
    }

    fn parse_project_config(&mut self, elem: &dom::Element) {
        for child in elem.children() {
            match child {
                ChildOfElement::Element(elem) => {
                    let name = elem.name().local_part();

                    if name != "ProjectConfiguration" {
                        // Error
                    }

                    match elem.attribute("Include") {
                        Some(attr) => {
                            let val = attr.value();
                            if self.config.is_empty() || self.config == val {
                                self.config = val.to_string();
                                self.traverse_element(&elem);
                            }
                        }
                        None => (), // Error
                    }
                }
                ChildOfElement::Text(_) => (),
                ChildOfElement::Comment(_) => (),
                ChildOfElement::ProcessingInstruction(_) => (),
            }
        }
    }

    fn is_project_config_item_group(&self, elem: &dom::Element) -> bool {
        if elem.name().local_part() == "ItemGroup" {
            match elem.attribute("Label") {
                Some(attr) => return attr.value() == "ProjectConfigurations",
                None => return false,
            }
        }

        return false;
    }

    fn traverse_element(&mut self, elem: &dom::Element) {
        if self.is_project_config_item_group(&elem) {
            self.parse_project_config(&elem);
        } else {
            for child in elem.children() {
                match child {
                    ChildOfElement::Element(elem) => self.traverse_element(&elem),
                    ChildOfElement::Text(text) => self.update_map(&elem, &text),
                    ChildOfElement::Comment(comment) => self.traverse_comment(&comment),
                    ChildOfElement::ProcessingInstruction(pi) => {
                        self.traverse_processing_instruction(&pi)
                    }
                };
            }
        }
    }

    fn traverse_text<'a>(&self, text: &'a dom::Text) -> &'a str {
        text.text().trim()
    }

    fn traverse_comment(&self, _comment: &dom::Comment) {}

    fn traverse_processing_instruction(&self, _pi: &dom::ProcessingInstruction) {}
}
