use std::result;
use thiserror::Error;
use xml::Xml;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error, PartialEq)]
pub enum ErrorKind {
    // -- CLI errors
    #[error("failed on HTTP GET")]
    HttpGet,

    #[error("failed to parse JSON")]
    ParseJson,

    #[error("source is not specified")]
    MissingSource,

    #[error("input format is not specified and failed to guess")]
    MissingInputFormat,

    #[error("failed to read from stdin")]
    ReadStdin,

    #[error("failed to read source")]
    ReadSource,

    #[error("failed to parse arguments")]
    InvalidArgument,

    // -- File system errors
    #[error("failed to read directory")]
    ReadDir,

    #[error("failed to read directory entry")]
    ReadDirEntry,

    #[error("failed to recursively create a directory")]
    CreateDirAll,

    #[error("failed to read a file")]
    ReadFile,

    #[error("failed to write a file")]
    WriteFile,

    #[error("there is no cache directory")]
    NoCacheDir,

    // -- Mintty errors
    #[error("invalid color representation: {0}")]
    InvalidColorFormat(String),

    #[error("invalid line: {0}")]
    InvalidLineFormat(String),

    #[error("unknown color name: {0}")]
    UnknownColorName(String),

    #[error("failed to parse int")]
    ParseInt,

    // -- iTerm errors
    #[error("invalid XML")]
    XMLParse,

    #[error("root dict was not found")]
    NoRootDict,

    #[error("cannot extract text from: {0}")]
    NotCharacterNode(Box<Xml>),

    #[error("unknown color component: {0}")]
    UnknownColorComponent(String),

    #[error("failed to parse float")]
    ParseFloat,

    // -- Provider errors
    #[error("unknown color scheme provider: {0}")]
    UnknownProvider(String),

    #[error("missing color scheme name")]
    MissingName,
}

pub struct WithKind<K> {
    pub kind: K,
    pub source: anyhow::Error,
}

pub trait Kind {
    type Ok;
    fn kind<K>(self, kind: K) -> std::result::Result<Self::Ok, WithKind<K>>;
}
