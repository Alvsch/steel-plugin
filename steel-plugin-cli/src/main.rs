use std::path::PathBuf;

use clap::{Parser, Subcommand};
use steel_plugin_cli::{PackageFormat, PackageOptions, package_plugin};

#[derive(Parser, Debug)]
#[command(
    name = "steel-plugin",
    version,
    about = "A CLI tool for steel-plugins",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Package a Luau plugin into a distributable archive
    Package {
        /// Path to the plugin directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output archive format
        #[arg(short, long, default_value = "tar.zst")]
        format: PackageFormat,

        /// Output directory
        #[arg(short, long, default_value = "target")]
        output: PathBuf,

        /// Enable release optimizations
        #[arg(long)]
        release: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Package {
            path,
            format,
            output,
            release,
        } => {
            println!(
                "Packaging {} as {format:?}, release={release}",
                path.display()
            );

            package_plugin(PackageOptions {
                input: path,
                output,
                format,
                release,
            })?;
        }
    }
    Ok(())
}
