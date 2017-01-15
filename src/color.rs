use std::num::ParseIntError;

// http://jadpole.github.io/rust/many-error-types
#[derive(Debug, PartialEq)]
pub enum ColorError {
    InvalidFormat,
    ParseInt(ParseIntError),
}

#[derive(Debug, Default, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn from_string(s: &str) -> Result<Self, ColorError> {
        let rgb: Vec<_> = s.split(",").collect();
        if rgb.len() != 3 {
            return Err(ColorError::InvalidFormat);
        }
        let red = Color::parse_int(rgb[0])?;
        let green = Color::parse_int(rgb[1])?;
        let blue = Color::parse_int(rgb[2])?;
        let color = Color { red: red, green: green, blue: blue };
        Ok(color)
    }

    fn parse_int(s: &str) -> Result<u8, ColorError> {
        s.parse().or_else(|e| Err(ColorError::ParseInt(e)))
    }

    pub fn to_hex(&self) -> String {
        format!("0x{:>02x}{:>02x}{:>02x}", self.red, self.green, self.blue)
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
    pub fn from_minttyrc(content: String) -> Self {
        let mut scheme = ColorScheme::default();
        for line in content.lines() {
            let components: Vec<&str> = line.split("=").collect();
            if components.len() != 2 {
                panic!("Invalid line: {}", line);
            }
            let name = components[0];
            let color = Color::from_string(components[1]).unwrap();
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
                _                  => panic!("Invalid color name: {}", name),
            }
        }
        scheme
    }

    pub fn to_yaml(&self) -> String {
        format!("colors:
  # Default colors
  primary:
    foreground: '{}'
    background: '{}'

  # Normal colors
  normal:
    black:      '{}'
    red:        '{}'
    green:      '{}'
    yellow:     '{}'
    blue:       '{}'
    magenta:    '{}'
    cyan:       '{}'
    white:      '{}'

  # Bright colors
  bright:
    black:      '{}'
    red:        '{}'
    green:      '{}'
    yellow:     '{}'
    blue:       '{}'
    magenta:    '{}'
    cyan:       '{}'
    white:      '{}'
",
            self.foreground.to_hex(),
            self.background.to_hex(),
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
