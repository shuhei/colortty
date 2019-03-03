extern crate colortty;
extern crate failure;
extern crate getopts;
extern crate json;
extern crate reqwest;

use colortty::*;
use failure::ResultExt;
use getopts::Options;
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
    let matches = opts
        .parse(&args[2..])
        .context(::ErrorKind::InvalidArgument)?;

    if matches.free.is_empty() {
        Err(::ErrorKind::MissingSource)?;
    }

    let source = &matches.free[0];
    let input_format = matches
        .opt_str("i")
        .and_then(|s| ColorSchemeFormat::from_string(&s))
        .or_else(|| ColorSchemeFormat::from_filename(&source))
        .ok_or(::ErrorKind::MissingInputFormat)?;

    let mut buffer = String::new();
    if source == "-" {
        io::stdin()
            .read_to_string(&mut buffer)
            .context(::ErrorKind::ReadStdin)?;
    } else {
        File::open(source)
            .unwrap()
            .read_to_string(&mut buffer)
            .context(::ErrorKind::ReadSource)?;
    }

    let scheme_result = match input_format {
        ColorSchemeFormat::ITerm => ColorScheme::from_iterm(&buffer),
        ColorSchemeFormat::Mintty => ColorScheme::from_minttyrc(&buffer),
    };

    scheme_result.map(|schema| println!("{}", schema.to_yaml()))
}

fn http_get(url: &str) -> ::Result<String> {
    // TODO: Use .json() with Deserialize?
    let client = reqwest::Client::new();
    let mut res = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "colortty")
        .send()
        .context(::ErrorKind::HttpGet)?;

    if !res.status().is_success() {
        Err(::ErrorKind::HttpGet)?
    }

    let body = res.text().context(::ErrorKind::HttpGet)?;
    Ok(body)
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

    ColorScheme::from_iterm(&body).map(|scheme| print!("{}", scheme.to_yaml()))
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
        return help();
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
        }
        _ => {}
    }
    Ok(())
}
