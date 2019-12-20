extern crate failure;
extern crate regex;
extern crate reqwest;
extern crate xml;

pub mod color;
pub mod error;
pub mod repo;

pub use crate::color::{Color, ColorScheme, ColorSchemeFormat};
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::repo::{http_get, Repo};
