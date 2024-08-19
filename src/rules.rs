use core::fmt::Debug;
// use enum_dispatch::enum_dispatch;
use declarative_enum_dispatch::enum_dispatch;

use crate::error::Result;
use crate::movy::{Layer, Pipe, Value};

#[derive(Debug, Clone)]
pub enum Rule {
    Filter(FilterEnum),
    Action(ActionEnum),
}

impl Rule {
    pub fn execute(&self, layer: &Layer, previous_rules: &Vec<Rule>) -> Result<(Layer, Vec<Rule>)> {
        match self {
            Self::Action(action) => action.execute(layer, previous_rules),
            Self::Filter(filter) => filter.execute(layer, previous_rules),
        }
    }
}

enum_dispatch!(
    pub trait Action {
        fn execute(&self, layer: &Layer, previous_rules: &Vec<Rule>) -> Result<(Layer, Vec<Rule>)>;
    }

    #[derive(Debug, Clone)]
    pub enum ActionEnum {
        Move(Move),
    }
);

enum_dispatch!(
    pub trait Filter {
        fn execute(&self, layer: &Layer, previous_rules: &Vec<Rule>) -> Result<(Layer, Vec<Rule>)>;
    }

    #[derive(Debug, Clone)]
    pub enum FilterEnum {
        Basename(Basename),
    }
);

#[derive(Debug, Clone)]
pub struct Basename {
    name: String,
    value: Value,
}

impl Basename {
    pub fn new(name: &str, value: Value) -> Rule {
        let filter: FilterEnum = (Basename {
            name: name.into(),
            value,
        })
        .into();
        Rule::Filter(filter)
    }
}

impl Filter for Basename {
    fn execute(&self, layer: &Layer, previous_rules: &Vec<Rule>) -> Result<(Layer, Vec<Rule>)> {
        println!("Executing Basename");

        let new_layer = layer.filter_map(layer.rule.clone(), |pipe| {
            let basename = pipe.path.file_name()?.to_string_lossy().to_string();

            match self.value.match_in(basename).ok()? {
                true => Some(pipe),
                false => None,
            }
        })?;

        return Ok((new_layer, vec![]));
    }
}

#[derive(Debug, Clone)]
pub struct Move {
    name: String,
    value: Value,
}

impl Move {
    pub fn new(name: &str, value: Value) -> Rule {
        let action = (Move {
            name: name.into(),
            value,
        })
        .into();
        Rule::Action(action)
    }
}

impl Action for Move {
    fn execute(&self, layer: &Layer, previous_rules: &Vec<Rule>) -> Result<(Layer, Vec<Rule>)> {
        println!("Executing Move");

        return Ok((layer.clone(), vec![]));
    }
}
