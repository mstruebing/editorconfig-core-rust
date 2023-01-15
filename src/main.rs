use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Specify conf filename other than '.editorconfig'
    #[arg(short, default_value_t = String::from(".editorconfig"))]
    f: String,

    /// Specify version (used by devs to test compatibility)
    #[arg(short, default_value_t = String::from(""))]
    b: String,

    /// Files to find configuration for.  Can be a hyphen (-) if you want path(s) to be read from stdin.
    #[arg(required(true))]
    filepath: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    println!("args: {:?}", args);
}
