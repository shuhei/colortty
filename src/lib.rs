extern crate async_std;
extern crate failure;
extern crate regex;
extern crate surf;
extern crate xml;

pub mod color;
pub mod error;

pub use crate::color::{Color, ColorScheme, ColorSchemeFormat};
pub use crate::error::{Error, ErrorKind, Result};
