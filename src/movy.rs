use derive_more::Constructor;
use regex::Regex;

use crate::error::{Error, Result};
use crate::rules;

use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Regex { content: String, flags: Vec<String> },
}

impl Value {
    pub fn match_in(&self, content: String) -> Result<bool> {
        match self {
            Self::String(value) => Ok(content.contains(value)),
            Self::Boolean(_) => Err(Error::Generic(
                "Cannot match string against boolean".to_string(),
            )),
            Self::Number(value) => Ok(content.contains(value.to_string().as_str())),
            Self::Regex {
                content: _,
                flags: _,
            } => {
                let re = self.build_regex()?;

                Ok(re.is_match(&content))
            }
        }
    }

    fn build_regex(&self) -> Result<Regex> {
        if let Self::Regex { content, flags } = self {
            let mut regex_string = String::from("");
            for flag in flags {
                regex_string.push_str(format!("(?{})", flag).as_str());
            }
            regex_string.push_str(content);

            return Ok(Regex::new(&regex_string)?);
        }

        return Err(Error::Generic(
            "Cannot build regex from invalid Value".to_string(),
        ));
    }
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub pipes: Vec<Pipe>,
    pub rule: Option<rules::Rule>,
}

impl Layer {
    pub fn filter_map(
        &self,
        rule: Option<rules::Rule>,
        closure: impl FnMut(Pipe) -> Option<Pipe>,
    ) -> Result<Layer> {
        let pipes = self
            .clone()
            .pipes
            .into_iter()
            .filter_map(closure)
            .collect::<Vec<Pipe>>();

        Ok(Layer { pipes, rule })
    }
}

#[derive(Debug, Constructor)]
pub struct Pipeline {
    name: String,
    properties: HashMap<String, Value>,
    pub rules: Vec<rules::Rule>,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Constructor)]
pub struct Movy {
    pub preamble: HashMap<String, Value>,
    pub pipelines: Vec<Pipeline>,
}

#[derive(Debug, Clone)]
pub struct Pipe {
    pub path: PathBuf,
}

impl Pipeline {
    pub fn layer_from_files(path: &str, with_dirs: bool) -> Result<Layer> {
        let pipes = fs::read_dir(path)?
            .filter_map(|f| {
                let Ok(file) = f else {
                    return None;
                };
                let Ok(metadata) = file.metadata() else {
                    return None;
                };
                if metadata.is_file() || with_dirs {
                    Some(Pipe { path: file.path() })
                } else {
                    None
                }
            })
            .collect::<Vec<Pipe>>();

        Ok(Layer { pipes, rule: None })
    }

    pub fn execute(self: &mut Self, input_layer: Layer) -> Result<Layer> {
        let mut i = 0;

        loop {
            let layer = self.layers.last().unwrap_or(&input_layer);
            let Some(rule) = self.rules.get(i) else {
                break;
            };

            let (new_layer, new_rules) = rule.execute(layer, &self.rules)?;

            self.rules.extend(new_rules);

            self.layers.push(new_layer);

            i += 1;
        }

        self.layers
            .last()
            .ok_or(Error::Generic(
                "Could not execute pipeline. Empty pipeline.".into(),
            ))
            .cloned()
    }
}
