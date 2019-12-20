extern crate colortty;
extern crate failure;
extern crate getopts;
extern crate json;

use colortty::{http_get, ColorScheme, ColorSchemeFormat, ErrorKind, Repo, Result};
use failure::ResultExt;
use getopts::Options;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return help();
    }

    let result = match args[1].as_ref() {
        "convert" => convert(args),
        "list" => list(args),
        "get" => get(args),
        "help" => help(),
        _ => {
            eprintln!("error: no such subcommand: `{}`", args[1]);
            process::exit(1);
        }
    };
    if let Err(e) = result {
        eprintln!("error: {}", e);
        process::exit(1);
    }
    Ok(())
}

// -- commands

fn convert(args: Vec<String>) -> Result<()> {
    let mut opts = Options::new();
    opts.optopt(
        "i",
        "input-format",
        "input format: 'iterm'|'mintty'|'gogh'",
        "INPUT_FORMAT",
    );
    let matches = opts.parse(&args[2..]).context(ErrorKind::InvalidArgument)?;

    if matches.free.is_empty() {
        return Err(ErrorKind::MissingSource.into());
    }

    let source = &matches.free[0];
    let input_format = matches
        .opt_str("i")
        .and_then(|s| ColorSchemeFormat::from_string(&s))
        .or_else(|| ColorSchemeFormat::from_filename(&source))
        .ok_or(ErrorKind::MissingInputFormat)?;

    let mut buffer = String::new();
    if source == "-" {
        io::stdin()
            .read_to_string(&mut buffer)
            .context(ErrorKind::ReadStdin)?;
    } else {
        File::open(source)
            .unwrap()
            .read_to_string(&mut buffer)
            .context(ErrorKind::ReadSource)?;
    }

    let scheme_result = match input_format {
        ColorSchemeFormat::ITerm => ColorScheme::from_iterm(&buffer),
        ColorSchemeFormat::Mintty => ColorScheme::from_minttyrc(&buffer),
        ColorSchemeFormat::Gogh => ColorScheme::from_gogh(&buffer),
    };

    scheme_result.map(|schema| println!("{}", schema.to_yaml()))
}

fn list(args: Vec<String>) -> Result<()> {
    let mut opts = Options::new();
    opts.optopt(
        "p",
        "provider",
        "color scheme provider: 'iterm'|'gogh'",
        "PROVIDER",
    );
    let matches = opts.parse(&args[2..]).context(ErrorKind::InvalidArgument)?;
    let provider = matches.opt_str("p").unwrap_or_else(|| "iterm".to_owned());
    match provider.as_ref() {
        "iterm" => list_iterm(),
        "gogh" => list_gogh(),
        _ => Err(ErrorKind::UnknownProvider(provider).into()),
    }
}

fn list_iterm() -> Result<()> {
    let schemes_url =
        "https://api.github.com/repos/mbadolato/iTerm2-Color-Schemes/contents/schemes";
    let buffer = http_get(schemes_url)?;
    let items = json::parse(&buffer).context(ErrorKind::ParseJson)?;
    for item in items.members() {
        let name = item["name"].as_str().unwrap().replace(".itermcolors", "");
        println!("{}", name);
    }
    Ok(())
}

fn list_gogh() -> Result<()> {
    let themes_url = "https://api.github.com/repos/Mayccoll/Gogh/contents/themes";
    let buffer = http_get(themes_url)?;
    let items = json::parse(&buffer).context(ErrorKind::ParseJson)?;
    for item in items.members() {
        let filename = item["name"].as_str().unwrap();
        if !filename.starts_with('_') && filename.ends_with(".sh") {
            let name = filename.replace(".sh", "");
            println!("{}", name);
        }
    }
    Ok(())
}

fn get(args: Vec<String>) -> Result<()> {
    let matches = parse_args_with_provider(args)?;

    if matches.free.is_empty() {
        return Err(ErrorKind::MissingName.into());
    }
    let name = &matches.free[0];

    let provider = matches.opt_str("p").unwrap_or_else(|| "iterm".to_owned());
    let color_scheme = match provider.as_ref() {
        "iterm" => {
            let repo = Repo::new("mbadolato", "iTerm2-Color-Schemes", "schemes");
            let body = repo.get(&format!("{}.itermcolors", name))?;
            ColorScheme::from_iterm(&body)
        }
        "gogh" => {
            let repo = Repo::new("Mayccoll", "Gogh", "themes");
            let body = repo.get(&format!("{}.sh", name))?;
            ColorScheme::from_gogh(&body)
        }
        _ => {
            return Err(ErrorKind::UnknownProvider(provider).into());
        }
    };
    color_scheme.map(|scheme| print!("{}", scheme.to_yaml()))
}

fn help() -> Result<()> {
    println!(
        "colortty - color scheme converter for alacritty

USAGE:
    # List color schemes at https://github.com/mbadolato/iTerm2-Color-Schemes
    colortty list
    colortty list -p iterm

    # List color schemes at https://github.com/Mayccoll/Gogh
    colortty list -p gogh

    # Get color scheme from https://github.com/mbadolato/iTerm2-Color-Schemes
    colortty get <color scheme name>
    colortty get -p iterm <color scheme name>

    # Get color scheme from https://github.com/Mayccoll/Gogh
    colortty get -p gogh <color scheme name>

    # Convert with implicit input type
    colortty convert some-color.itermcolors
    colortty convert some-color.minttyrc
    colortty convert some-color.sh

    # Convert with explicit input type
    colortty convert -i iterm some-color-theme
    colortty convert -i mintty some-color-theme
    colortty convert -i gogh some-color-theme

    # Convert stdin (explicit input type is necessary)
    cat some-color-theme | colortty convert -i iterm -
    cat some-color-theme | colortty convert -i mintty -
    cat some-color-theme | colortty convert -i gogh -"
    );

    Ok(())
}

// -- Utility functions

fn parse_args_with_provider(args: Vec<String>) -> Result<getopts::Matches> {
    let mut opts = Options::new();
    opts.optopt(
        "p",
        "provider",
        "color scheme provider: 'iterm'|'gogh'",
        "PROVIDER",
    );
    let matches = opts.parse(&args[2..]).context(ErrorKind::InvalidArgument)?;
    Ok(matches)
}
