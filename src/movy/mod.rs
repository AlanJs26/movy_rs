pub mod expression;
pub mod pipeline;

use derive_more::Constructor;
pub use expression::Value;
pub use pipeline::Pipeline;

use std::collections::HashMap;

use crate::cli::Cli;
use crate::error;
use crate::error::Result;

#[derive(Debug, Constructor)]
pub struct Movy {
    pub preamble: HashMap<String, Value>,
    pub pipelines: Vec<Pipeline>,
}

#[derive(Debug, Clone)]
pub struct Preamble {
    pub simulate: bool,
    pub quiet: bool,
    pub root: String,
}

impl Preamble {
    pub fn new(preamble: &HashMap<String, Value>, cli: &Cli) -> Result<Preamble> {
        let simulate = match (preamble.get("simulate"), &cli.simulate) {
            (_, Some(value)) => *value,
            (Some(Value::Boolean(value)), None) => *value,
            _ => false,
        };
        let quiet = match (preamble.get("quiet"), &cli.quiet) {
            (_, Some(value)) => *value,
            (Some(Value::Boolean(value)), None) => *value,
            _ => false,
        };

        let root = match (preamble.get("root"), &cli.root) {
            (_, Some(root)) => Ok(root.clone()),
            (Some(Value::String(root)), None) => Ok(root.clone()),
            _ => Err(error::Error::Generic("Missing root. You must either specify the root inside the file preamble or override with --path flag".into())),
        }?;

        return Ok(Preamble {
            simulate,
            quiet,
            root,
        });
    }
}
