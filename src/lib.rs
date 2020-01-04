pub mod color;
pub mod error;
pub mod provider;

pub use crate::color::{Color, ColorScheme, ColorSchemeFormat};
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::provider::Provider;
