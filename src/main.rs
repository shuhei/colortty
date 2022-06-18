use anyhow::{Context, Result};
use colortty::{ColorScheme, ColorSchemeFormat, Provider};
use getopts::Options;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

#[async_std::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return help();
    }

    match args[1].as_ref() {
        "convert" => handle_error(convert(args)),
        "list" => handle_error(list(args).await),
        "get" => handle_error(get(args).await),
        "help" => help(),
        _ => {
            eprintln!("error: no such subcommand: `{}`", args[1]);
            process::exit(1);
        }
    };
}

fn handle_error(result: Result<()>) {
    if let Err(e) = result {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CliError {
    #[error("source is not specified")]
    MissingSource,

    #[error("input format is not specified and failed to guess")]
    MissingInputFormat,

    #[error("failed to read from stdin")]
    ReadStdin,

    #[error("failed to read source")]
    ReadSource,

    #[error("failed to parse arguments")]
    InvalidArgument,

    #[error("unknown color scheme provider: {0}")]
    UnknownProvider(String),

    #[error("missing color scheme name")]
    MissingName,
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
    let matches = opts.parse(&args[2..]).context(CliError::InvalidArgument)?;

    if matches.free.is_empty() {
        return Err(CliError::MissingSource.into());
    }

    let source = &matches.free[0];
    let input_format = matches
        .opt_str("i")
        .and_then(|s| ColorSchemeFormat::from_string(&s))
        .or_else(|| ColorSchemeFormat::from_filename(&source))
        .ok_or(CliError::MissingInputFormat)?;

    let mut buffer = String::new();
    if source == "-" {
        io::stdin()
            .read_to_string(&mut buffer)
            .context(CliError::ReadStdin)?;
    } else {
        File::open(source)
            .unwrap()
            .read_to_string(&mut buffer)
            .context(CliError::ReadSource)?;
    }

    let scheme_result = match input_format {
        ColorSchemeFormat::ITerm => ColorScheme::from_iterm(&buffer),
        ColorSchemeFormat::Mintty => ColorScheme::from_minttyrc(&buffer),
        ColorSchemeFormat::Gogh => ColorScheme::from_gogh(&buffer),
    };

    scheme_result.map(|schema| println!("{}", schema.to_yaml()))
}

async fn list(args: Vec<String>) -> Result<()> {
    let mut opts = Options::new();
    set_provider_option(&mut opts);
    opts.optflag("u", "update-cache", "update color scheme cache");

    let matches = opts.parse(&args[2..]).context(CliError::InvalidArgument)?;
    let provider = get_provider(&matches)?;

    if matches.opt_present("u") {
        provider.download_all().await?;
    }

    let color_schemes = provider.list().await?;

    let mut max_name_length = 0;
    for (name, _) in &color_schemes {
        max_name_length = max_name_length.max(name.len());
    }

    for (name, color_scheme) in &color_schemes {
        println!(
            "{:width$} {}",
            name,
            color_scheme.to_preview(),
            width = max_name_length
        );
    }

    Ok(())
}

async fn get(args: Vec<String>) -> Result<()> {
    let mut opts = Options::new();
    set_provider_option(&mut opts);
    let matches = opts.parse(&args[2..]).context(CliError::InvalidArgument)?;

    if matches.free.is_empty() {
        return Err(CliError::MissingName.into());
    }
    let name = &matches.free[0].to_string();

    let provider = get_provider(&matches)?;
    let color_scheme = provider.get(name).await?;
    print!("# {}\n{}", name, color_scheme.to_yaml());

    Ok(())
}

fn help() {
    println!(
        "colortty - color scheme converter for alacritty

USAGE:
    # List color schemes at https://github.com/mbadolato/iTerm2-Color-Schemes
    colortty list
    colortty list -p iterm
    colortty list -u # update cached color schemes

    # List color schemes at https://github.com/Mayccoll/Gogh
    colortty list -p gogh
    colortty list -p gogh -u # update cached color schemes

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
}

// -- Utility functions

fn set_provider_option(opts: &mut getopts::Options) {
    opts.optopt(
        "p",
        "provider",
        "color scheme provider: 'iterm'|'gogh'",
        "PROVIDER",
    );
}

fn get_provider(matches: &getopts::Matches) -> Result<Provider> {
    let provider_name = matches.opt_str("p").unwrap_or_else(|| "iterm".to_owned());
    let provider = match provider_name.as_ref() {
        "iterm" => Provider::iterm(),
        "gogh" => Provider::gogh(),
        _ => return Err(CliError::UnknownProvider(provider_name).into()),
    };
    Ok(provider)
}
