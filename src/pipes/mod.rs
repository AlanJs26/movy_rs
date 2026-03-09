mod basename_filter;
mod move_action;
use core::fmt::Debug;
use declarative_enum_dispatch::enum_dispatch;

use crate::error::Result;
use crate::movy::pipeline::PipeOutput;
use crate::movy::Preamble;
pub use basename_filter::Basename;
pub use move_action::Move;

pub struct PipeInfo {
    pub name: String,
    pub span: String,
}

#[derive(Debug, Clone)]
pub enum Pipe {
    Filter(FilterEnum),
    Action(ActionEnum),
}

impl Pipe {
    pub fn execute(
        &self,
        layer: &PipeOutput,
        preamble: &Preamble,
    ) -> Result<(PipeOutput, Vec<Pipe>)> {
        match self {
            Self::Action(action) => action.execute(layer, preamble),
            Self::Filter(filter) => filter.execute(layer, preamble),
        }
    }
    pub fn info(&self) -> PipeInfo {
        match self {
            Self::Action(action) => action.info(),
            Self::Filter(filter) => filter.info(),
        }
    }
}

enum_dispatch!(
    pub trait Action {
        fn execute(
            &self,
            layer: &PipeOutput,
            preamble: &Preamble,
        ) -> Result<(PipeOutput, Vec<Pipe>)>;
        fn info(&self) -> PipeInfo;
    }

    #[derive(Debug, Clone)]
    pub enum ActionEnum {
        Move(Move),
    }
);

enum_dispatch!(
    pub trait Filter {
        fn execute(
            &self,
            layer: &PipeOutput,
            preamble: &Preamble,
        ) -> Result<(PipeOutput, Vec<Pipe>)>;
        fn info(&self) -> PipeInfo;
    }

    #[derive(Debug, Clone)]
    pub enum FilterEnum {
        Basename(Basename),
    }
);
