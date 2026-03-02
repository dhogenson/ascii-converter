mod cli;
mod core;
mod utilities;

use anyhow::Result;
use cli::args::validate_args;
use core::image_to_ascii::render_image_to_ascii;

fn main() -> Result<()> {
    let args = validate_args()?;
    render_image_to_ascii(&args.input, &args.output)?;

    Ok(())
}
