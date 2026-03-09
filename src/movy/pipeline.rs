use crate::movy::expression::Value;
use crate::movy::Preamble;
use crate::pipes;
use crate::{error::Result, pipes::Pipe};
use derive_more::Constructor;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct PipeOutput {
    pub items: Vec<Item>,
}

impl PipeOutput {
    pub fn filter_map(&self, closure: impl FnMut(Item) -> Option<Item>) -> Result<PipeOutput> {
        let pipes = self
            .clone()
            .items
            .into_iter()
            .filter_map(closure)
            .collect::<Vec<Item>>();

        Ok(PipeOutput { items: pipes })
    }
}

#[derive(Debug, Constructor)]
pub struct Pipeline {
    pub name: String,
    pub properties: HashMap<String, Value>,
    pub pipes: Vec<pipes::Pipe>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub path: PathBuf,
}

impl Pipeline {
    pub fn input_from_files(path: &str, with_dirs: bool) -> Result<PipeOutput> {
        let pipes = fs::read_dir(path)?
            .filter_map(|f| {
                let Ok(file) = f else {
                    return None;
                };
                let Ok(metadata) = file.metadata() else {
                    return None;
                };
                if metadata.is_file() || with_dirs {
                    Some(Item { path: file.path() })
                } else {
                    None
                }
            })
            .collect::<Vec<Item>>();

        Ok(PipeOutput { items: pipes })
    }

    pub fn execute(self: &mut Self, input: PipeOutput, preamble: &Preamble) -> Result<PipeOutput> {
        self.execute_with_processors(input, preamble, None, None)
    }

    pub fn execute_with_processors(
        self: &mut Self,
        input: PipeOutput,
        preamble: &Preamble,
        before_fn: Option<fn(&PipeOutput, &Pipe)>,
        after_fn: Option<fn(&PipeOutput, &Pipe)>,
    ) -> Result<PipeOutput> {
        let mut i = 0;
        let mut output: PipeOutput = input;

        loop {
            let Some(pipe) = self.pipes.get(i) else {
                break;
            };

            if let Some(f) = &before_fn {
                f(&output, &pipe);
            };
            let (new_output, new_rules) = pipe.execute(&output, preamble)?;
            if let Some(f) = &after_fn {
                f(&new_output, &pipe);
            };

            self.pipes.extend(new_rules);

            output = new_output;
            i += 1;
        }

        Ok(output)
    }
}
