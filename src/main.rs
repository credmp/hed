use std::process::exit;

pub(crate) use color_eyre::eyre::Result;
use errors::ApplicationError;
use termion::color;
use clap::Parser;

use crate::hostfile::HostFile;
pub mod app;
pub mod errors;
pub mod hostentry;
pub mod hostfile;
pub mod utils;

use app::Commands;

fn main() {
    if let Err(e) = color_eyre::install() {
        eprintln!("Could not setup error handling: {}", e);
        exit(exits::RUNTIME_ERROR);
    }

    let matches = app::Cli::parse();

    let mut hf = HostFile {
        filename: matches.file, 
        entries: None,
    };

    if let Err(e) = hf.parse() {
        eprintln!("Failed to parse the hostfile, this should not happen unless you are using --file to override the file. The error message is: {}", e);
        exit(exits::RUNTIME_ERROR);
    }

    let res: Result<(), ApplicationError> = match matches.command {
        Commands::Verify {} => {
            verify(hf)
        },
        Commands::Show {  } => {
            hf.show()
        },
        Commands::Add { hostname, ip } => {
            match hf.add(hostname, ip) {
                Ok(()) => hf.write(),
                Err(e) => {
                    eprintln!("Failed to process command: {}", e);
                    exit(exits::RUNTIME_ERROR);
                }
            }            
        },
        Commands::Replace { hostname, ip } => {
            match hf.replace(hostname, ip) {
                Ok(()) => hf.write(),
                Err(e) => {
                    eprintln!("Failed to process command: {}", e);
                    exit(exits::RUNTIME_ERROR);
                }
            }            
        },
        Commands::Delete { entry } => {
            match hf.delete(entry) {
                Ok(()) => hf.write(),
                Err(e) => {
                    eprintln!("Failed to process command: {}", e);
                    exit(exits::RUNTIME_ERROR);
                }
            }            
                
        },
        Commands::Import { filename, dry_run} => {
            match hf.import(filename) {
                Ok(_) => {
                    if dry_run {
                        hf.show()                        
                    } else {
                        hf.write()
                    }
                },
                Err(e) => {
                    Err(e)
                }
            }
        },
    };

    match res {
        Ok(_) => {
            exit(exits::SUCCESS);
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(exits::RUNTIME_ERROR);
        }
    };
}

/// Verify that the host file is parsable
fn verify(hf: HostFile) -> Result<(), ApplicationError> {
    println!(
        "Hostsfile is readable and contains {}{}{} entries.",
        color::Fg(color::Green),
        hf.entries.unwrap_or_else(Vec::new).len(),
        color::Fg(color::Reset),
    );
    Ok(())
}

mod exits {

    /// Exit code for when exa runs OK.
    pub const SUCCESS: i32 = 0;

    /// Exit code for when there was at least one I/O error during execution.
    pub const RUNTIME_ERROR: i32 = 1;
}
