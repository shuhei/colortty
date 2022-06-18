use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ErrorKind {
    // -- CLI errors
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
}
