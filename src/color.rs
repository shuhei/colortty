use xml::{Element, Xml};
use failure::ResultExt;

pub enum ColorSchemeFormat {
    ITerm,
    Mintty,
}

impl ColorSchemeFormat {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "iterm"  => Some(ColorSchemeFormat::ITerm),
            "mintty" => Some(ColorSchemeFormat::Mintty),
            _        => None,
        }
    }

    pub fn from_filename(s: &str) -> Option<Self> {
        if s.contains(".itermcolors") {
            return Some(ColorSchemeFormat::ITerm);
        } else if s.contains(".minttyrc") {
            return Some(ColorSchemeFormat::Mintty);
        } else {
            return None;
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    // TODO: Move this out because it's only for mintty format.
    pub fn from_string(s: &str) -> ::Result<Self> {
        let rgb: Vec<_> = s.split(",").collect();
        if rgb.len() != 3 {
            Err(::ErrorKind::InvalidColorFormat(s.to_owned()))?;
        }
        let red = parse_int(rgb[0])?;
        let green = parse_int(rgb[1])?;
        let blue = parse_int(rgb[2])?;
        let color = Color { red: red, green: green, blue: blue };
        Ok(color)
    }

    pub fn to_hex(&self) -> String {
        format!("0x{:>02x}{:>02x}{:>02x}", self.red, self.green, self.blue)
    }
}

fn parse_int(s: &str) -> ::Result<u8> {
    Ok(s.parse::<u8>().context(::ErrorKind::ParseInt)?)
}

fn extract_text(element: &Element) -> ::Result<&str> {
    let first = &element.children[0];
    match first {
        &Xml::CharacterNode(ref text) => Ok(text),
        _ => Err(::ErrorKind::NotCharacterNode(first.to_owned()))?,
    }
}

#[derive(Default)]
pub struct ColorScheme {
    foreground: Color,
    background: Color,

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
    pub fn from_minttyrc(content: &str) -> ::Result<Self> {
        let mut scheme = ColorScheme::default();
        for line in content.lines() {
            let components: Vec<&str> = line.split("=").collect();
            if components.len() != 2 {
                Err(::ErrorKind::InvalidLineFormat(line.to_owned()))?;
            }
            let name = components[0];
            let color = Color::from_string(components[1])?;
            match name {
                "ForegroundColour" => scheme.foreground     = color,
                "BackgroundColour" => scheme.background     = color,
                "Black"            => scheme.black          = color,
                "Red"              => scheme.red            = color,
                "Green"            => scheme.green          = color,
                "Yellow"           => scheme.yellow         = color,
                "Blue"             => scheme.blue           = color,
                "Magenta"          => scheme.magenta        = color,
                "Cyan"             => scheme.cyan           = color,
                "White"            => scheme.white          = color,
                "BoldRed"          => scheme.bright_red     = color,
                "BoldBlack"        => scheme.bright_black   = color,
                "BoldGreen"        => scheme.bright_green   = color,
                "BoldYellow"       => scheme.bright_yellow  = color,
                "BoldBlue"         => scheme.bright_blue    = color,
                "BoldMagenta"      => scheme.bright_magenta = color,
                "BoldCyan"         => scheme.bright_cyan    = color,
                "BoldWhite"        => scheme.bright_white   = color,
                _                  => {
                    Err(::ErrorKind::UnknownColorName(name.to_owned()))?
                },
            }
        }
        Ok(scheme)
    }

    pub fn from_iterm(content: &str) -> ::Result<Self> {
        let mut scheme = ColorScheme::default();

        let root = content.parse::<Element>().context(::ErrorKind::XMLParse)?;
        let root_dict: &Element = root.get_children("dict", None).nth(0)
            .ok_or(::ErrorKind::NoRootDict)?;

        let keys = root_dict.get_children("key", None);
        let values = root_dict.get_children("dict", None);
        for (key, value) in keys.zip(values) {
            let color_name = extract_text(key)?;
            let color_keys = value.get_children("key", None);
            let color_values = value.get_children("real", None);

            let mut color = Color::default();
            for (color_key, color_value) in color_keys.zip(color_values) {
                let component_name = extract_text(color_key)?;
                let real_value: f32 = extract_text(color_value)?
                    .parse::<f32>()
                    .context(::ErrorKind::ParseFloat)?;
                let int_value = (real_value * 255.0) as u8;
                match component_name {
                    "Red Component"   => color.red   = int_value,
                    "Green Component" => color.green = int_value,
                    "Blue Component"  => color.blue  = int_value,
                    "Alpha Component" => {},
                    "Color Space"     => {},
                    _                 => {
                        Err(::ErrorKind::UnknownColorComponent(component_name.to_owned()))?;
                    },
                };
            }

            match color_name {
                "Ansi 0 Color"     => scheme.black          = color,
                "Ansi 1 Color"     => scheme.red            = color,
                "Ansi 2 Color"     => scheme.green          = color,
                "Ansi 3 Color"     => scheme.yellow         = color,
                "Ansi 4 Color"     => scheme.blue           = color,
                "Ansi 5 Color"     => scheme.magenta        = color,
                "Ansi 6 Color"     => scheme.cyan           = color,
                "Ansi 7 Color"     => scheme.white          = color,
                "Ansi 8 Color"     => scheme.bright_black   = color,
                "Ansi 9 Color"     => scheme.bright_red     = color,
                "Ansi 10 Color"    => scheme.bright_green   = color,
                "Ansi 11 Color"    => scheme.bright_yellow  = color,
                "Ansi 12 Color"    => scheme.bright_blue    = color,
                "Ansi 13 Color"    => scheme.bright_magenta = color,
                "Ansi 14 Color"    => scheme.bright_cyan    = color,
                "Ansi 15 Color"    => scheme.bright_white   = color,
                "Background Color" => scheme.background     = color,
                "Foreground Color" => scheme.foreground     = color,
                _                  => (),
            }
        }

        Ok(scheme)
    }

    pub fn to_yaml(&self) -> String {
        format!("colors:
  # Default colors
  primary:
    background: '{}'
    foreground: '{}'

  # Normal colors
  normal:
    black:   '{}'
    red:     '{}'
    green:   '{}'
    yellow:  '{}'
    blue:    '{}'
    magenta: '{}'
    cyan:    '{}'
    white:   '{}'

  # Bright colors
  bright:
    black:   '{}'
    red:     '{}'
    green:   '{}'
    yellow:  '{}'
    blue:    '{}'
    magenta: '{}'
    cyan:    '{}'
    white:   '{}'
",
            self.background.to_hex(),
            self.foreground.to_hex(),
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
}
