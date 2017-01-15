extern crate colortty;

#[cfg(test)]
mod color_tests {
    mod color {
        use colortty::color::{Color, ColorError};

        #[test]
        fn from_string_works() {
            assert_eq!(
                Color::from_string("12,3,255").unwrap(),
                Color { red: 12, green: 3, blue: 255 }
            );
        }

        #[test]
        fn from_string_invalid_format() {
            assert_eq!(
                Color::from_string("123"),
                Err(ColorError::InvalidFormat)
            );
        }

        #[test]
        fn from_string_parse_int_error() {
            assert!(Color::from_string("abc,3,fo").is_err());
        }

        #[test]
        fn to_hex() {
            assert_eq!(
                Color { red: 123, green: 4, blue: 255 }.to_hex(),
                "0x7b04ff"
            );
        }
    }

    mod color_scheme {
        use colortty::color::{ColorScheme};

        #[test]
        fn convert_dracula() {
            let dracula_minttyrc = "ForegroundColour=248,248,242
BackgroundColour=40,42,54
Black=0,0,0
BoldBlack=40,42,53
Red=255,85,85
BoldRed=255,110,103
Green=80,250,123
BoldGreen=90,247,142
Yellow=241,250,140
BoldYellow=244,249,157
Blue=202,169,250
BoldBlue=202,169,250
Magenta=255,121,198
BoldMagenta=255,146,208
Cyan=139,233,253
BoldCyan=154,237,254
White=191,191,191
BoldWhite=230,230,230".to_string();
            let dracula_alacritty = "colors:
  # Default colors
  primary:
    background: '0x282a36'
    foreground: '0xf8f8f2'

  # Normal colors
  normal:
    black:   '0x000000'
    red:     '0xff5555'
    green:   '0x50fa7b'
    yellow:  '0xf1fa8c'
    blue:    '0xcaa9fa'
    magenta: '0xff79c6'
    cyan:    '0x8be9fd'
    white:   '0xbfbfbf'

  # Bright colors
  bright:
    black:   '0x282a35'
    red:     '0xff6e67'
    green:   '0x5af78e'
    yellow:  '0xf4f99d'
    blue:    '0xcaa9fa'
    magenta: '0xff92d0'
    cyan:    '0x9aedfe'
    white:   '0xe6e6e6'
";
            let scheme = ColorScheme::from_minttyrc(dracula_minttyrc);
            assert_eq!(scheme.to_yaml(), dracula_alacritty);
        }
    }
}
