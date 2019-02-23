extern crate colortty;
extern crate failure;
extern crate getopts;
extern crate hyper;
extern crate hyper_openssl;
extern crate json;

use colortty::*;
use failure::ResultExt;
use getopts::Options;
use hyper::client::Client;
use hyper::header::UserAgent;
use hyper::net::HttpsConnector;
use hyper_openssl::OpensslClient;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

fn convert(args: Vec<String>) -> ::Result<()> {
    let mut opts = Options::new();
    opts.optopt(
        "i",
        "input-format",
        "input format: 'iterm'|'mintty'",
        "INPUT_FORMAT",
    );
    let matches = match opts.parse(&args[2..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.free.is_empty() {
        panic!("Specify source");
    }

    let source = &matches.free[0];
    let input_format = matches
        .opt_str("i")
        .and_then(|s| ColorSchemeFormat::from_string(&s))
        .or_else(|| ColorSchemeFormat::from_filename(&source))
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

    let scheme_result = match input_format {
        ColorSchemeFormat::ITerm => ColorScheme::from_iterm(&buffer),
        ColorSchemeFormat::Mintty => ColorScheme::from_minttyrc(&buffer),
    };

    match scheme_result {
        Ok(schema) => println!("{}", schema.to_yaml()),
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
    Ok(())
}

fn http_get(url: &str) -> ::Result<String> {
    let ssl = OpensslClient::new().context(::ErrorKind::HttpGet)?;
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    let mut res = client
        .get(url)
        .header(UserAgent("colortty".to_string()))
        .send()
        .context(::ErrorKind::HttpGet)?;
    let mut buffer = String::new();
    // TODO: Check status code.
    res.read_to_string(&mut buffer).context(::ErrorKind::HttpGet)?;
    Ok(buffer)
}

fn list() -> ::Result<()> {
    // TODO: Get only necessary fields.
    let schemes_url =
        "https://api.github.com/repos/mbadolato/iTerm2-Color-Schemes/contents/schemes";
    let buffer = http_get(schemes_url)?;
    let items = json::parse(&buffer).context(::ErrorKind::ParseJson)?;
    for item in items.members() {
        let name = item["name"].as_str().unwrap().replace(".itermcolors", "");
        println!("{}", name);
    }
    Ok(())
}

fn get(args: Vec<String>) -> ::Result<()> {
    let name = &args[2];
    let url = format!("https://raw.githubusercontent.com/mbadolato/iTerm2-Color-Schemes/master/schemes/{}.itermcolors", name);
    let body = http_get(&url)?;

    match ColorScheme::from_iterm(&body) {
        Ok(scheme) => print!("{}", scheme.to_yaml()),
        Err(e) => panic!(format!("{:?}", e)),
    }
    Ok(())
}

fn help() -> Result<()> {
    println!(
        "colortty - color scheme converter for alacritty

USAGE:
    # List color schemes at https://github.com/mbadolato/iTerm2-Color-Schemes
    colortty list

    # Get color scheme from https://github.com/mbadolato/iTerm2-Color-Schemes
    colortty get <color scheme name>

    # Convert with implicit input type
    colortty convert some-color.itermcolors
    colortty convert some-color.minttyrc

    # Convert with explicit input type
    colortty convert -i iterm some-color-theme
    colortty convert -i mintty some-color-theme

    # Convert stdin (explicit input type is necessary)
    cat some-color-theme | colortty convert -i iterm -
    cat some-color-theme | colortty convert -i mintty -"
    );

    Ok(())
}

fn main() -> ::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        help();
        return Ok(());
    }

    let result = match args[1].as_ref() {
        "convert" => convert(args),
        "list" => list(),
        "get" => get(args),
        "help" => help(),
        _ => {
            eprintln!("error: no such subcommand: `{}`", args[1]);
            process::exit(1);
        }
    };
    match result {
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        },
        _ => {},
    }
    Ok(())
}
