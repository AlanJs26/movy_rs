use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(version, about, arg_required_else_help = true)]
pub struct Cli {
    pub file: String,

    /// List all pipelines
    #[arg(short, long)]
    pub list: bool,

    /// Print as little information as possible
    #[arg(short, long, action=ArgAction::SetTrue)]
    pub quiet: Option<bool>,

    /// Do not apply actions
    #[arg(short, long, action=ArgAction::SetTrue)]
    pub simulate: Option<bool>,

    /// Run specific pipeline
    #[arg(short, long)]
    pub pipeline: Option<String>,

    /// Override the working directory
    #[arg(short, long)]
    pub root: Option<String>,
}
