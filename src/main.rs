mod error;
mod movy;
mod parsing;
mod rules;

use movy::{Movy, Pipeline};

fn main() {
    let mut movy = match Movy::from_file("/mnt/DiscoExterno/_Codes/Rust/movy_rs/src/test.movy") {
        Ok(result) => result,
        Err(error) => {
            println!("Error:\n");
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    println!("{:#?}", movy);

    let pipeline = movy.pipelines.first_mut().unwrap();

    let first_layer = Pipeline::layer_from_files("./", true).unwrap();
    println!("{:#?}", first_layer);

    let output_layer = pipeline.execute(first_layer);
    println!("{:#?}", output_layer);
}
