mod app;
mod cli;
mod analyzer;
mod ui;
mod config;
mod document_processor;

#[cfg(test)]
mod test_git;

use anyhow::Result;
use clap::{Parser, CommandFactory};

use crate::app::App;
use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Some(cmd) => {
            let mut app = App::new().await?;
            app.run_command(cmd).await?;
        }
        None => {
            // Default to showing help when no command is specified
            Cli::command().print_help()?;
        }
    }
    
    Ok(())
}