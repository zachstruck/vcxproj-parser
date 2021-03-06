use crate::condition;
use encoding_rs::UTF_8;
use regex::Captures;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use sxd_document::dom;
use sxd_document::dom::ChildOfElement;
use sxd_document::dom::ChildOfRoot;
use sxd_document::parser;

pub struct Vcxproj {
    pub values: HashMap<String, String>,
    config: String,
}

impl Vcxproj {
    pub fn new() -> Vcxproj {
        Vcxproj {
            values: HashMap::new(),
            config: String::new(),
        }
    }

    pub fn read_vcxproj<P: AsRef<Path>>(
        &mut self,
        filename: P,
    ) -> Result<(), condition::ParseError> {
        // msbuild variables
        {
            // MSBuildThisFile
            self.values.insert(
                "MSBuildThisFile".to_string(),
                filename
                    .as_ref()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );

            // MSBuildThisFileDirectory
            let mut dir = filename
                .as_ref()
                .parent()
                .unwrap()
                .to_path_buf()
                .to_str()
                .unwrap()
                .to_string();
            if !dir.ends_with(std::path::MAIN_SEPARATOR) {
                dir.push(std::path::MAIN_SEPARATOR);
            }
            self.values
                .insert("MSBuildThisFileDirectory".to_string(), dir);

            // TODO
            // MSBuildThisFileDirectoryNoRoot

            // MSBuildThisFileExtension
            self.values.insert(
                "MSBuildThisFileExtension".to_string(),
                filename
                    .as_ref()
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );

            // MSBuildThisFileFullPath
            self.values.insert(
                "MSBuildThisFileFullPath".to_string(),
                filename.as_ref().to_str().unwrap().to_string(),
            );

            // MSBuildThisFileName
            self.values.insert(
                "MSBuildThisFileName".to_string(),
                filename
                    .as_ref()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
        }

        let raw_text = fs::read_to_string(filename).unwrap();
        let vcxproj = UTF_8.decode_with_bom_removal(raw_text.as_bytes());

        // FIXME
        // Should this signal an error
        // if there were replacement characters?
        assert!(!vcxproj.1);

        let vcxproj = vcxproj.0;

        let package = parser::parse(&vcxproj).unwrap();
        let document = package.as_document();

        let root = document.root();
        for child in root.children() {
            match child {
                ChildOfRoot::Element(elem) => self.traverse_element(&elem)?,
                ChildOfRoot::Comment(comment) => self.traverse_comment(&comment),
                ChildOfRoot::ProcessingInstruction(pi) => self.traverse_processing_instruction(&pi),
            };
        }

        Ok(())
    }

    fn update_map(&mut self, elem: &dom::Element, text: &dom::Text) {
        let val = self.traverse_text(&text);
        if !val.is_empty() {
            let key = elem.name().local_part();
            match self
                .values
                .insert(key.to_string(), self.resolve_variables(val))
            {
                Some(old) => println!("Key: {} | Replacing \"{}\" with \"{}\"", key, old, val),
                None => (),
            }
        }
    }

    fn resolve_variables(&self, s: &str) -> String {
        // FIXME
        // Use the lazy_static! or something
        // to avoid recompiling this regular expression
        let re = Regex::new(r"\$\(([[:alnum:]_]*)\)").unwrap();
        re.replace_all(s, |caps: &Captures| {
            assert!(caps.len() == 2);
            let cap = &caps[1];
            let replaced = match self.values.get(cap) {
                Some(val) => val.to_string(),
                None => match env::var(&cap) {
                    Ok(val) => val.to_string(),
                    Err(_) => {
                        println!("Unable to resolve variable: $({})", cap);
                        String::new()
                    }
                },
            };
            // FIXME
            // Recursively replace variables until the base case is found
            // This needs some kind of cycle detection to prevent infinite recursion
            if re.is_match(&replaced) {
                self.resolve_variables(&replaced)
            } else {
                replaced
            }
        })
        .to_string()
    }

    fn failed_condition(&mut self, elem: &dom::Element) -> Result<bool, condition::ParseError> {
        match elem.attribute("Condition") {
            Some(attr) => {
                let raw_cond = dbg!(attr.value());
                let cond = dbg!(self.resolve_variables(raw_cond));
                Ok(!condition::eval_condition(&cond)?)
            }
            None => Ok(false),
        }
    }

    fn parse_project_config(&mut self, elem: &dom::Element) -> Result<(), condition::ParseError> {
        assert!(elem.name().local_part() == "ItemGroup");
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
                                self.traverse_element(&elem)?;
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

        Ok(())
    }

    fn is_project_config_item_group(&self, elem: &dom::Element) -> bool {
        if elem.name().local_part() == "ItemGroup" {
            match elem.attribute("Label") {
                Some(attr) => return attr.value() == "ProjectConfigurations",
                None => return false,
            }
        }

        false
    }

    fn parse_import(&mut self, elem: &dom::Element) -> Result<(), condition::ParseError> {
        assert!(elem.name().local_part() == "Import");
        match elem.attribute("Project") {
            Some(attr) => {
                let filename = self.resolve_variables(attr.value());
                self.read_vcxproj(filename)?;
            }
            // FIXME  Error
            None => unreachable!(),
        }

        Ok(())
    }

    fn is_import(&self, elem: &dom::Element) -> bool {
        if elem.name().local_part() == "Import" {
            match elem.attribute("Project") {
                Some(_) => return true,
                None => return false,
            }
        }

        false
    }

    fn traverse_element(&mut self, elem: &dom::Element) -> Result<(), condition::ParseError> {
        if self.failed_condition(&elem)? {
            return Ok(());
        }

        if self.is_project_config_item_group(&elem) {
            self.parse_project_config(&elem)?;
        } else if self.is_import(&elem) {
            self.parse_import(&elem)?;
        } else {
            for child in elem.children() {
                match child {
                    ChildOfElement::Element(elem) => self.traverse_element(&elem)?,
                    ChildOfElement::Text(text) => self.update_map(&elem, &text),
                    ChildOfElement::Comment(comment) => self.traverse_comment(&comment),
                    ChildOfElement::ProcessingInstruction(pi) => {
                        self.traverse_processing_instruction(&pi)
                    }
                };
            }
        }

        Ok(())
    }

    fn traverse_text<'a>(&self, text: &'a dom::Text) -> &'a str {
        text.text().trim()
    }

    fn traverse_comment(&self, _comment: &dom::Comment) {}

    fn traverse_processing_instruction(&self, _pi: &dom::ProcessingInstruction) {}
}
