use anyhow::{Context, Result};
use regex::Regex;
use xml::{Element, Xml};

pub enum ColorSchemeFormat {
    ITerm,
    Mintty,
    Gogh,
}

impl ColorSchemeFormat {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "iterm" => Some(ColorSchemeFormat::ITerm),
            "mintty" => Some(ColorSchemeFormat::Mintty),
            "gogh" => Some(ColorSchemeFormat::Gogh),
            _ => None,
        }
    }

    pub fn from_filename(s: &str) -> Option<Self> {
        if s.ends_with(".itermcolors") {
            Some(ColorSchemeFormat::ITerm)
        } else if s.ends_with(".minttyrc") {
            Some(ColorSchemeFormat::Mintty)
        } else if s.ends_with(".sh") {
            Some(ColorSchemeFormat::Gogh)
        } else {
            None
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseError {
    // -- Generic errors
    #[error("failed to parse int")]
    ParseInt,

    #[error("failed to parse float")]
    ParseFloat,

    // -- Mintty parse errors
    #[error("invalid color representation: {0}")]
    InvalidColorFormat(String),

    #[error("invalid line: {0}")]
    InvalidLineFormat(String),

    #[error("unknown color name: {0}")]
    UnknownColorName(String),

    // -- iTerm parse errors
    #[error("invalid XML")]
    XMLParse,

    #[error("root dict was not found")]
    NoRootDict,

    #[error("cannot extract text from: {0}")]
    NotCharacterNode(Box<Xml>),

    #[error("unknown color component: {0}")]
    UnknownColorComponent(String),
}

#[derive(Debug, Default, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn from_mintty_color(s: &str) -> Result<Self> {
        let rgb: Vec<_> = s.split(',').collect();
        if rgb.len() != 3 {
            return Err(ParseError::InvalidColorFormat(s.to_owned()).into());
        }
        let red = parse_int(rgb[0])?;
        let green = parse_int(rgb[1])?;
        let blue = parse_int(rgb[2])?;
        Ok(Color { red, green, blue })
    }

    pub fn from_gogh_color(s: &str) -> Result<Self> {
        let red = parse_hex(&s[1..3])?;
        let green = parse_hex(&s[3..5])?;
        let blue = parse_hex(&s[5..7])?;
        Ok(Color { red, green, blue })
    }

    pub fn to_hex(&self) -> String {
        format!("0x{:>02x}{:>02x}{:>02x}", self.red, self.green, self.blue)
    }

    pub fn to_24bit_be(&self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.red, self.green, self.blue)
    }

    pub fn to_24bit_preview(&self) -> String {
        format!("\x1b[38;2;{};{};{}m●", self.red, self.green, self.blue)
    }
}

fn parse_int(s: &str) -> Result<u8> {
    Ok(s.parse::<u8>().context(ParseError::ParseInt)?)
}

fn parse_hex(s: &str) -> Result<u8> {
    Ok(u8::from_str_radix(s, 16).context(ParseError::ParseInt)?)
}

fn extract_text(element: &Element) -> Result<&str> {
    let first = &element.children[0];
    match first {
        Xml::CharacterNode(ref text) => Ok(text),
        _ => Err(ParseError::NotCharacterNode(Box::new(first.to_owned())).into()),
    }
}

fn extract_real_color(element: &Element) -> Result<u8> {
    let real_value = extract_text(element)?
        .parse::<f32>()
        .context(ParseError::ParseFloat)?;
    let int_value = (real_value * 255.0) as u8;
    Ok(int_value)
}

#[derive(Default)]
pub struct ColorScheme {
    foreground: Color,
    background: Color,
    cursor_text: Option<Color>,
    cursor: Option<Color>,

    black: Color,
    red: Color,
    green: Color,
    yellow: Color,
    blue: Color,
    magenta: Color,
    cyan: Color,
    white: Color,

    bright_black: Color,
    bright_red: Color,
    bright_green: Color,
    bright_yellow: Color,
    bright_blue: Color,
    bright_magenta: Color,
    bright_cyan: Color,
    bright_white: Color,
}

impl ColorScheme {
    // From a mintty color theme (.minttyrc)
    pub fn from_minttyrc(content: &str) -> Result<Self> {
        let mut scheme = ColorScheme::default();
        for line in content.lines() {
            let components: Vec<&str> = line.split('=').collect();
            if components.len() != 2 {
                return Err(ParseError::InvalidLineFormat(line.to_owned()).into());
            }
            let name = components[0];
            let color = Color::from_mintty_color(components[1])?;
            match name {
                "ForegroundColour" => scheme.foreground = color,
                "BackgroundColour" => scheme.background = color,
                "Black" => scheme.black = color,
                "Red" => scheme.red = color,
                "Green" => scheme.green = color,
                "Yellow" => scheme.yellow = color,
                "Blue" => scheme.blue = color,
                "Magenta" => scheme.magenta = color,
                "Cyan" => scheme.cyan = color,
                "White" => scheme.white = color,
                "BoldRed" => scheme.bright_red = color,
                "BoldBlack" => scheme.bright_black = color,
                "BoldGreen" => scheme.bright_green = color,
                "BoldYellow" => scheme.bright_yellow = color,
                "BoldBlue" => scheme.bright_blue = color,
                "BoldMagenta" => scheme.bright_magenta = color,
                "BoldCyan" => scheme.bright_cyan = color,
                "BoldWhite" => scheme.bright_white = color,
                _ => return Err(ParseError::UnknownColorName(name.to_owned()).into()),
            }
        }
        Ok(scheme)
    }

    // From an iTerm 2 color theme (.itermcolors)
    pub fn from_iterm(content: &str) -> Result<Self> {
        let mut scheme = ColorScheme::default();

        let root = content.parse::<Element>().context(ParseError::XMLParse)?;
        let root_dict: &Element = root
            .get_children("dict", None)
            .nth(0)
            .ok_or(ParseError::NoRootDict)?;

        let keys = root_dict.get_children("key", None);
        let values = root_dict.get_children("dict", None);
        for (key, value) in keys.zip(values) {
            let color_name = extract_text(key)?;

            let mut color = Color::default();
            // Extract element pairs like <key/><real/><key/><real/><key/><real/>
            // `element.get_children()` doesn't work well here because there might be
            //  a pattern like <key/><real/><key/><string/><key/><real/>.
            //  In this case, we want to ignore the second pair (<key/><string/>).
            let element_nodes = value
                .children
                .iter()
                .flat_map(|child| match child {
                    Xml::ElementNode(elem) => Some(elem),
                    _ => None,
                })
                .collect::<Vec<_>>();
            for pair in element_nodes.chunks(2) {
                if let [color_key, color_value] = pair {
                    let component_name = extract_text(color_key)?;
                    match component_name {
                        "Red Component" => color.red = extract_real_color(color_value)?,
                        "Green Component" => color.green = extract_real_color(color_value)?,
                        "Blue Component" => color.blue = extract_real_color(color_value)?,
                        "Alpha Component" => {}
                        "Color Space" => {}
                        _ => {
                            return Err(ParseError::UnknownColorComponent(
                                component_name.to_owned(),
                            )
                            .into());
                        }
                    };
                }
            }

            match color_name {
                "Ansi 0 Color" => scheme.black = color,
                "Ansi 1 Color" => scheme.red = color,
                "Ansi 2 Color" => scheme.green = color,
                "Ansi 3 Color" => scheme.yellow = color,
                "Ansi 4 Color" => scheme.blue = color,
                "Ansi 5 Color" => scheme.magenta = color,
                "Ansi 6 Color" => scheme.cyan = color,
                "Ansi 7 Color" => scheme.white = color,
                "Ansi 8 Color" => scheme.bright_black = color,
                "Ansi 9 Color" => scheme.bright_red = color,
                "Ansi 10 Color" => scheme.bright_green = color,
                "Ansi 11 Color" => scheme.bright_yellow = color,
                "Ansi 12 Color" => scheme.bright_blue = color,
                "Ansi 13 Color" => scheme.bright_magenta = color,
                "Ansi 14 Color" => scheme.bright_cyan = color,
                "Ansi 15 Color" => scheme.bright_white = color,
                "Background Color" => scheme.background = color,
                "Foreground Color" => scheme.foreground = color,
                "Cursor Color" => scheme.cursor = Some(color),
                "Cursor Text Color" => scheme.cursor_text = Some(color),
                _ => (),
            }
        }

        Ok(scheme)
    }

    // From a gogh color theme file (.sh)
    pub fn from_gogh(content: &str) -> Result<Self> {
        // Match against export XXX="yyy"
        let pattern = Regex::new(r#"export ([A-Z0-9_]+)="(#[0-9a-fA-F]{6})""#).unwrap();
        let mut scheme = ColorScheme::default();
        for line in content.lines() {
            if let Some(caps) = pattern.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let color = Color::from_gogh_color(caps.get(2).unwrap().as_str())?;
                match name {
                    "FOREGROUND_COLOR" => scheme.foreground = color,
                    "BACKGROUND_COLOR" => scheme.background = color,
                    "COLOR_01" => scheme.black = color,
                    "COLOR_02" => scheme.red = color,
                    "COLOR_03" => scheme.green = color,
                    "COLOR_04" => scheme.yellow = color,
                    "COLOR_05" => scheme.blue = color,
                    "COLOR_06" => scheme.magenta = color,
                    "COLOR_07" => scheme.cyan = color,
                    "COLOR_08" => scheme.white = color,
                    "COLOR_09" => scheme.bright_black = color,
                    "COLOR_10" => scheme.bright_red = color,
                    "COLOR_11" => scheme.bright_green = color,
                    "COLOR_12" => scheme.bright_yellow = color,
                    "COLOR_13" => scheme.bright_blue = color,
                    "COLOR_14" => scheme.bright_magenta = color,
                    "COLOR_15" => scheme.bright_cyan = color,
                    "COLOR_16" => scheme.bright_white = color,
                    _ => {}
                }
            }
        }
        Ok(scheme)
    }

    // Output TOML that can be used as a color theme in .alacritty.toml
    pub fn to_toml(&self) -> String {
        let cursor_colors = match (&self.cursor_text, &self.cursor) {
            (Some(cursor_text), Some(cursor)) => format!(
                "
# Cursor colors
[colors.cursor]
text =   '{}'
cursor = '{}'
",
                cursor_text.to_hex(),
                cursor.to_hex()
            ),
            _ => String::new(),
        };

        format!(
            "
# Default colors
[colors.primary]
background = '{}'
foreground = '{}'
{}
# Normal colors
[colors.normal]
black =   '{}'
red =     '{}'
green =   '{}'
yellow =  '{}'
blue =    '{}'
magenta = '{}'
cyan =    '{}'
white =   '{}'

# Bright colors
[colors.bright]
black =   '{}'
red =     '{}'
green =   '{}'
yellow =  '{}'
blue =    '{}'
magenta = '{}'
cyan =    '{}'
white =   '{}'
",
            self.background.to_hex(),
            self.foreground.to_hex(),
            cursor_colors,
            self.black.to_hex(),
            self.red.to_hex(),
            self.green.to_hex(),
            self.yellow.to_hex(),
            self.blue.to_hex(),
            self.magenta.to_hex(),
            self.cyan.to_hex(),
            self.white.to_hex(),
            self.bright_black.to_hex(),
            self.bright_red.to_hex(),
            self.bright_green.to_hex(),
            self.bright_yellow.to_hex(),
            self.bright_blue.to_hex(),
            self.bright_magenta.to_hex(),
            self.bright_cyan.to_hex(),
            self.bright_white.to_hex(),
        )
    }

    // Show all colors in one line
    pub fn to_preview(&self) -> String {
        let colors = vec![
            self.background.to_24bit_be(),
            " ".to_string(),
            self.foreground.to_24bit_preview(),
            "  ".to_string(),
            self.black.to_24bit_preview(),
            self.red.to_24bit_preview(),
            self.green.to_24bit_preview(),
            self.yellow.to_24bit_preview(),
            self.blue.to_24bit_preview(),
            self.magenta.to_24bit_preview(),
            self.cyan.to_24bit_preview(),
            self.white.to_24bit_preview(),
            "  ".to_string(),
            self.bright_black.to_24bit_preview(),
            self.bright_red.to_24bit_preview(),
            self.bright_green.to_24bit_preview(),
            self.bright_yellow.to_24bit_preview(),
            self.bright_blue.to_24bit_preview(),
            self.bright_magenta.to_24bit_preview(),
            self.bright_cyan.to_24bit_preview(),
            self.bright_white.to_24bit_preview(),
            " ".to_string(),
            "\x1b[0m".to_string(),
        ];
        colors.join("")
    }
}
