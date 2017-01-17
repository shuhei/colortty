extern crate getopts;
extern crate hyper;
extern crate hyper_openssl;
extern crate colortty;
#[macro_use]
extern crate json;

use std::env;
use std::io::{self, Read};
use std::fs::File;
use getopts::Options;
use hyper::client::Client;
use hyper::net::HttpsConnector;
use hyper::header::{UserAgent};
use hyper_openssl::OpensslClient;
use colortty::color::{ColorScheme, ColorSchemeFormat};

fn convert(args: Vec<String>) {
    let mut opts = Options::new();
    opts.optopt("i", "input-format", "input format: 'iterm'|'mintty'", "INPUT_FORMAT");
    let matches = match opts.parse(&args[2..]) {
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

fn list() {
    let ssl = OpensslClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    // TODO: Get only necessary fields.
    let schemes_url = "https://api.github.com/repos/mbadolato/iTerm2-Color-Schemes/contents/schemes";

    let mut res = client.get(schemes_url)
        .header(UserAgent("colortty".to_string()))
        .send()
        .unwrap();
    let mut json = String::new();
    // TODO: Check status code.
    res.read_to_string(&mut json).unwrap();
    let parsed = json::parse(&json).unwrap();
    for item in parsed.members() {
        let name = item["name"].as_str().unwrap().replace(".itermcolors", "");
        println!("{}", name);
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    match args[1].as_ref() {
        "convert" => convert(args),
        "list"    => list(),
        _         => panic!(format!("{}", args[1])),
    }
}
