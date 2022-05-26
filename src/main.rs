use std::process::exit;

use clap::Parser;
pub(crate) use color_eyre::eyre::Result;
use errors::ApplicationError;
use termion::color;
use utils::Modifications;

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

    let res: Result<Modifications, ApplicationError> = match matches.command {
        Commands::Verify {} => verify(hf),
        Commands::Show {} => hf.show(),
        Commands::Add { hostname, ip } => match hf.add(hostname, ip) {
            Ok(m) => {
                let r = hf.write();
                if r.is_err() {
                    Err(r.err().unwrap())
                } else {
                    Ok(m)
                }
            }

            Err(e) => {
                eprintln!("Failed to process command: {}", e);
                exit(exits::RUNTIME_ERROR);
            }
        },
        Commands::Replace { hostname, ip } => match hf.replace(hostname, ip) {
            Ok(m) => {
                let r = hf.write();
                if r.is_err() {
                    Err(r.err().unwrap())
                } else {
                    Ok(m)
                }
            }
            Err(e) => {
                eprintln!("Failed to process command: {}", e);
                exit(exits::RUNTIME_ERROR);
            }
        },
        Commands::Delete { entry } => match hf.delete(entry) {
            Ok(m) => {
                let r = hf.write();
                if r.is_err() {
                    Err(r.err().unwrap())
                } else {
                    Ok(m)
                }
            }
            Err(e) => {
                eprintln!("Failed to process command: {}", e);
                exit(exits::RUNTIME_ERROR);
            }
        },
    };

    match res {
        Ok(m) => {
            print_status(m);
            exit(exits::SUCCESS);
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(exits::RUNTIME_ERROR);
        }
    };
}

fn print_status(mods: Modifications) {
    if mods.added_entries > 0 {
        println!(
            "Added {}{}{} entries",
            color::Fg(color::Green),
            mods.added_entries,
            color::Fg(color::Reset)
        )
    }
    if mods.updated_entries > 0 {
        println!(
            "Updated {}{}{} entries",
            color::Fg(color::Green),
            mods.updated_entries,
            color::Fg(color::Reset)
        )
    }
    if mods.removed_entries > 0 {
        println!(
            "Removed {}{}{} entries",
            color::Fg(color::Green),
            mods.removed_entries,
            color::Fg(color::Reset)
        )
    }
}

/// Verify that the host file is parsable
fn verify(hf: HostFile) -> Result<Modifications, ApplicationError> {
    println!(
        "Hostsfile is readable and contains {}{}{} entries.",
        color::Fg(color::Green),
        hf.entries.unwrap_or_default().len(),
        color::Fg(color::Reset),
    );
    Ok(Modifications::new())
}

mod exits {

    /// Exit code for when exa runs OK.
    pub const SUCCESS: i32 = 0;

    /// Exit code for when there was at least one I/O error during execution.
    pub const RUNTIME_ERROR: i32 = 1;
}
