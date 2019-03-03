extern crate failure;
extern crate reqwest;
extern crate xml;

pub mod color;
pub mod error;

pub use color::{ColorScheme, ColorSchemeFormat};
pub use error::{Error, ErrorKind, Result};
