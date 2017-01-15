extern crate getopts;
extern crate colortty;

use std::env;
use std::io::{self, Read};
use getopts::Options;
use colortty::color::{ColorScheme};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.reqopt("i", "input", "input file format", "INPUT_FORMAT");
    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(f) => panic!(f.to_string()),
    };
    let input_format = matches.opt_str("i").unwrap();

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Failed to get stdin");

    let scheme = match input_format.as_ref() {
        "iterm2" => ColorScheme::from_iterm2(buffer),
        "mintty" => ColorScheme::from_minttyrc(buffer),
        _        => panic!("Unsupported format: {}", input_format),
    };

    print!("{}", scheme.to_yaml());
}
