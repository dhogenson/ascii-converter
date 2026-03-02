use anyhow::Result;
use clap::Parser;
use std::fs;

#[derive(Parser)]
#[command(name = "ascii-convert")]
#[command(about = "Convert images to ASCII art")]
pub struct Args {
    /// Input image file path
    pub input: String,

    /// Output image file path
    pub output: String,
}

pub fn validate_args() -> Result<Args> {
    let args = Args::parse();

    if !fs::exists(&args.input)? {
        return Err(anyhow::anyhow!("Input file does not exist"));
    }

    Ok(args)
}
