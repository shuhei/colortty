use std::convert::From;
use std::fmt::{self, Display};
use std::result;
use failure::{Backtrace, Context, Fail};
use xml::Xml;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail, PartialEq)]
pub enum ErrorKind {
    // -- Mintty errors

    #[fail(display="invalid color representation: {}", _0)]
    InvalidColorFormat(String),

    #[fail(display="invalid line: {}", _0)]
    InvalidLineFormat(String),

    #[fail(display="unknown color name: {}", _0)]
    UnknownColorName(String),

    #[fail(display="failed to parse int")]
    ParseInt,

    // -- iTerm errors

    #[fail(display="invalid XML")]
    XMLParse,

    #[fail(display="root dict was not found")]
    NoRootDict,

    #[fail(display="cannot extract text from: {}", _0)]
    NotCharacterNode(Xml),

    #[fail(display="unknown color component: {}", _0)]
    UnknownColorComponent(String),

    #[fail(display="failed to parse float")]
    ParseFloat,
}

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &*self.inner.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        let inner = Context::new(kind);
        Error { inner }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}
