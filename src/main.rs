#[macro_use]
extern crate clap;
use clap::App;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let _ = App::from_yaml(yaml).get_matches();
}
