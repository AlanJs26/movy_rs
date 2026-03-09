mod cli;
mod error;
mod movy;
mod parsing;
mod pipes;

use clap::Parser;
use movy::{Movy, Pipeline};

use crate::cli::Cli;
use crate::movy::{pipeline::PipeOutput, Preamble};
use error::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // let mut movy = match Movy::from_file("tests/syntax/test.movy") {
    let mut movy = match Movy::from_file(&cli.file) {
        Ok(result) => result,
        Err(error) => {
            println!("Error:\n");
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    let preamble = Preamble::new(&movy.preamble, &cli)?;

    if cli.list == true {
        for pipeline in movy.pipelines {
            println!("{}", pipeline.name)
        }
        return Ok(());
    }

    if let Some(pipeline_name) = cli.pipeline {
        let pipeline = movy
            .pipelines
            .iter_mut()
            .find(|p| p.name == pipeline_name)
            .unwrap_or_else(|| {
                println!("Error:\n");
                eprintln!(
                    "{}",
                    error::Error::Generic(format!("Pipeline \"{}\" does not exist", pipeline_name))
                );
                std::process::exit(1);
            });

        run_pipeline(pipeline, preamble)?;
        return Ok(());
    }

    for pipeline in &mut movy.pipelines {
        run_pipeline(pipeline, preamble.clone())?;
    }
    return Ok(());
}

fn run_pipeline(pipeline: &mut Pipeline, preamble: Preamble) -> Result<PipeOutput> {
    let first_layer = Pipeline::input_from_files(&preamble.root, true)?;

    println!("Running \"{}\"", pipeline.name);
    println!("Path: {}\n", preamble.root);

    if preamble.quiet {
        pipeline.execute(first_layer, &preamble)
    } else {
        println!("input files:");
        print_items(&first_layer.items);

        pipeline.execute_with_processors(
            first_layer,
            &preamble,
            Some(|_, pipe| {
                println!("\nRunning \"{}\"", pipe.info().span);
            }),
            Some(|output, _| {
                print_items(&output.items);
            }),
        )
    }
}

fn print_items(items: &Vec<movy::pipeline::Item>) {
    for item in items {
        let path_str = item
            .path
            .clone()
            .into_os_string()
            .into_string()
            .unwrap_or_default();
        println!("{}", path_str);
    }
}
