use pest;
use pest::Span;
use pest::error::ErrorVariant;

use crate::parsing::Rule;
pub type Result<T> = core::result::Result<T, Error>;

use derive_more::{From, Display};

// pub type Error = Box<dyn std::error::Error>;

#[derive(Debug, From, Display)]
pub enum Error {
    Generic(String),

    #[from]
    Parsing(pest::error::Error<Rule>),

    #[from]
    Io(std::io::Error),

    #[from]
    Regex(regex::Error),
}

impl Error {
    pub fn from_span(message: String, span: Span) -> Error {
        Error::Parsing(
            pest::error::Error::new_from_span(
                ErrorVariant::CustomError { message },
                span
            )
        )
    }
}

