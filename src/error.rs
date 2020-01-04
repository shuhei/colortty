use failure::{Backtrace, Context, Fail};
use std::convert::From;
use std::fmt::{self, Display};
use std::result;
use xml::Xml;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail, PartialEq)]
pub enum ErrorKind {
    // -- CLI errors
    #[fail(display = "failed on HTTP GET")]
    HttpGet,

    #[fail(display = "failed to parse JSON")]
    ParseJson,

    #[fail(display = "source is not specified")]
    MissingSource,

    #[fail(display = "input format is not specified and failed to guess")]
    MissingInputFormat,

    #[fail(display = "failed to read from stdin")]
    ReadStdin,

    #[fail(display = "failed to read source")]
    ReadSource,

    #[fail(display = "failed to parse arguments")]
    InvalidArgument,

    // -- Git errors
    #[fail(display = "failed to clone a repository")]
    GitClone,

    #[fail(display = "failed to check out a repository")]
    GitCheckout,

    #[fail(display = "failed to revparse")]
    GitRevparse,

    #[fail(display = "failed to get a tree in a repository")]
    GitGetTree,

    // -- File system errors
    #[fail(display = "failed to read directory")]
    ReadDir,

    #[fail(display = "failed to read directory entry")]
    ReadDirEntry,

    #[fail(display = "failed to recursively create a directory")]
    CreateDirAll,

    #[fail(display = "there is no cache directory")]
    NoCacheDir,

    // -- Mintty errors
    #[fail(display = "invalid color representation: {}", _0)]
    InvalidColorFormat(String),

    #[fail(display = "invalid line: {}", _0)]
    InvalidLineFormat(String),

    #[fail(display = "unknown color name: {}", _0)]
    UnknownColorName(String),

    #[fail(display = "failed to parse int")]
    ParseInt,

    // -- iTerm errors
    #[fail(display = "invalid XML")]
    XMLParse,

    #[fail(display = "root dict was not found")]
    NoRootDict,

    #[fail(display = "cannot extract text from: {}", _0)]
    NotCharacterNode(Box<Xml>),

    #[fail(display = "unknown color component: {}", _0)]
    UnknownColorComponent(String),

    #[fail(display = "failed to parse float")]
    ParseFloat,

    // -- Provider errors
    #[fail(display = "unknown color scheme provider: {}", _0)]
    UnknownProvider(String),

    #[fail(display = "missing color scheme name")]
    MissingName,
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
    fn cause(&self) -> Option<&dyn Fail> {
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
