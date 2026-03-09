use crate::error::Result;
use crate::movy::Preamble;
use crate::pipes::{Filter, FilterEnum, PipeInfo};
use crate::{movy::expression::Value, movy::pipeline::PipeOutput, pipes::Pipe};

#[derive(Debug, Clone)]
pub struct Basename {
    name: String,
    value: Value,
    span: String,
}

impl Basename {
    pub fn new(name: &str, value: Value, span: String) -> Pipe {
        let filter: FilterEnum = (Basename {
            name: name.into(),
            value,
            span,
        })
        .into();
        Pipe::Filter(filter)
    }
}

impl Filter for Basename {
    fn execute(&self, input: &PipeOutput, preamble: &Preamble) -> Result<(PipeOutput, Vec<Pipe>)> {
        let new_layer = input.filter_map(|item| {
            let basename = item.path.file_name()?.to_string_lossy().to_string();

            match self.value.match_in(basename).ok()? {
                true => Some(item),
                false => None,
            }
        })?;

        return Ok((new_layer, vec![]));
    }
    fn info(&self) -> super::PipeInfo {
        PipeInfo {
            name: self.name.clone(),
            span: self.span.clone(),
        }
    }
}
