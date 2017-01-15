extern crate colortty;

use std::io::{self, Read};
use colortty::color::{ColorScheme};

fn main() {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Failed to get stdin");

    let scheme = ColorScheme::from_minttyrc(buffer);
    print!("{}", scheme.to_yaml());
}
