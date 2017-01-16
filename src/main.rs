extern crate getopts;
extern crate colortty;

use std::env;
use std::io::{self, Read};
use std::fs::File;
use getopts::Options;
use colortty::color::{ColorScheme, ColorSchemeFormat};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("i", "input-format", "input format: 'iterm'|'mintty'", "INPUT_FORMAT");
    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.free.is_empty() {
        panic!("Specify source");
    }

    let source = &matches.free[0];
    let input_format = matches.opt_str("i")
        .and_then(|s| ColorSchemeFormat::from_string(s.as_ref()))
        .or_else(|| ColorSchemeFormat::from_filename(source.as_ref()))
        .expect("Input format not specified and failed to guess");

    let mut buffer = String::new();
    if source == "-" {
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to get stdin");
    } else {
        File::open(source)
            .unwrap()
            .read_to_string(&mut buffer)
            .expect("Failed to read source");
    }

    let scheme = match input_format {
        ColorSchemeFormat::ITerm => ColorScheme::from_iterm(&buffer),
        ColorSchemeFormat::Mintty => ColorScheme::from_minttyrc(&buffer),
    };

    print!("{}", scheme.to_yaml());
}
