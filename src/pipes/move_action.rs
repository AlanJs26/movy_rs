use crate::error::Result;
use crate::movy::Preamble;
use crate::pipes::PipeInfo;
use crate::{
    movy::expression::Value,
    movy::pipeline::PipeOutput,
    pipes::{Action, Pipe},
};

#[derive(Debug, Clone)]
pub struct Move {
    name: String,
    value: Value,
    span: String,
}

impl Move {
    pub fn new(name: &str, value: Value, span: String) -> Pipe {
        let action = (Move {
            name: name.into(),
            value,
            span,
        })
        .into();
        Pipe::Action(action)
    }
}

impl Action for Move {
    fn execute(&self, layer: &PipeOutput, preamble: &Preamble) -> Result<(PipeOutput, Vec<Pipe>)> {
        if preamble.simulate {
            println!("Executing Move (Simulation)");
        } else {
            println!("Executing Move");
        }

        return Ok((layer.clone(), vec![]));
    }
    fn info(&self) -> super::PipeInfo {
        PipeInfo {
            name: self.name.clone(),
            span: self.span.clone(),
        }
    }
}
