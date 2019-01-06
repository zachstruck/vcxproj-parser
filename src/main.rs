#[macro_use]
extern crate clap;
extern crate sxd_document;
use clap::App;
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

    let root = document.root();
    for child in root.children() {
        match child {
            ChildOfRoot::Element(elem) => traverse_element(&elem),
            ChildOfRoot::Comment(comment) => traverse_comment(&comment),
            ChildOfRoot::ProcessingInstruction(pi) => traverse_processing_instruction(&pi),
        };
    }
}

fn traverse_element(elem: &dom::Element) {
    println!("{:?}", elem.name());
    for child in elem.children() {
        match child {
            ChildOfElement::Element(elem) => traverse_element(&elem),
            ChildOfElement::Text(text) => traverse_text(&text),
            ChildOfElement::Comment(comment) => traverse_comment(&comment),
            ChildOfElement::ProcessingInstruction(pi) => (traverse_processing_instruction(&pi)),
        };
    }
}

fn traverse_text(text: &dom::Text) {
    let text = text.text().trim();
    if !text.is_empty() {
        println!("{}", text);
    }
}

fn traverse_comment(comment: &dom::Comment) {
    println!("{}", comment.text());
}

fn traverse_processing_instruction(pi: &dom::ProcessingInstruction) {
    match pi.value() {
        Some(val) => println!("{}", val),
        None => (),
    }
}
