use std::net::IpAddr;

use clap::{App, Arg};
use color_eyre::eyre::eyre;
pub use color_eyre::eyre::Result;
use termion::color;

use crate::{hostentry::HostEntry, hostfile::HostFile};

pub mod hostentry;
pub mod hostfile;
pub mod utils;

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Host EDitor")
        .long_about("Host EDitor allows you to maniuplate the /etc/hosts file. It will manage adding new hosts and removing old entries. Any entry added will be validated (valid ip, non-existing previous entry).")
        .setting(clap::AppSettings::ColoredHelp)
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::new("file")
                .long("file")
                .required(false)
                .takes_value(true)
                .about("Instead of /etc/hosts, use this file (testing)"),
        )
        .subcommand(
            App::new("verify")
                .about("Verify the integrity of the hosts file")
                .version("0.1"),
        )
        .subcommand(
            App::new("show")
                .about("List your current hostfile")
                .version("0.1"),
        )
        .subcommand(
            App::new("add")
                .about("Add a host to your hostfile")
                .version("0.1")
                .arg(
                    Arg::new("hostname")
                        .index(1)
                        .required(true)
                        .about("Hostname to add to the hostfile"),
                )
                .arg(
                    Arg::new("ip")
                        .index(2)
                        .required(false)
                        .about("Optional: ip address of the hostname"),
                ),
        )
        .subcommand(
            App::new("delete")
                .about("Delete a host from your hostfile")
                .version("0.1")
                .arg(
                    Arg::new("entry")
                        .index(1)
                        .required(true)
                        .about("IP or Hostname to remove"),
                ),
        )
        .get_matches();

    let res = match matches.subcommand_name() {
        Some("verify") => verify(get_filename(&matches)),
        Some("show") => show(get_filename(&matches)),
        Some("add") => {
            let mymatches = matches
                .subcommand_matches("add")
                .expect("Cannot be a subcommand and not be a subcommand");

            let ip = if mymatches.is_present("ip") {
                mymatches.value_of("ip")
            } else {
                None
            };
            add(get_filename(&matches), mymatches.value_of("hostname"), ip)
        }
        Some("delete") => {
            let mymatches = matches
                .subcommand_matches("delete")
                .expect("Cannot be a subcommand and not be a subcommand");

            delete(get_filename(&matches), mymatches.value_of("entry"))
        }
        Some(x) => {
            println!("Unimplemented command {} called", x);
            unimplemented!()
        }
        _ => {
            println!("No subcommand given");
            unimplemented!()
        }
    };
    res
}

fn get_filename(matches: &clap::ArgMatches) -> &str {
    match matches.value_of("file") {
        Some(x) => x,
        _ => "/etc/hosts",
    }
}

fn add(filename: &str, hostname: Option<&str>, ip: Option<&str>) -> Result<(), color_eyre::Report> {
    let ip_address: Option<IpAddr> = match ip {
        Some(x) => match x.parse() {
            Ok(y) => Some(y),
            Err(e) => return Err(eyre!(e)),
        },
        _ => None,
    };

    if hostname.is_none() {
        return Err(eyre!("No hostname given"));
    }

    let mut hf = HostFile {
        filename: filename.to_string(),
        entries: None,
    };

    if let Err(x) = hf.parse() {
        return Err(eyre!("Failed to parse file {}", x));
    }

    // if IP address is given, find a matching hostentry to add a alias
    //    no ip? add new entry
    // if only a name is given, find a HostEntry already serving a tld
    //    if none are found, err
    if let Some(ip_a) = ip_address {
        for item in hf.entries.iter_mut().flatten() {
            let i = item;

            if i.has_ip(&ip_a) && !i.has_name(hostname.unwrap()) {
                i.add_alias(hostname.unwrap());
                if let Err(x) = hf.write() {
                    return Err(eyre!("Error: {}", x));
                }
                return Ok(());
            }
        }
        hf.add_host_entry(HostEntry {
            ip: ip_address,
            name: Some(hostname.unwrap().to_string()),
            comment: None,
            aliasses: None,
        });
        if let Err(x) = hf.write() {
            return Err(eyre!("Error: {}", x));
        }
        Ok(())
    } else {
        for item in hf.entries.iter_mut().flatten() {
            let i = item;

            if i.can_resolve_host(hostname.unwrap()) && !i.has_name(hostname.unwrap()) {
                i.add_alias(hostname.unwrap());
                if let Err(x) = hf.write() {
                    return Err(eyre!("Error: {}", x));
                }
                return Ok(());
            }
        }

        Err(eyre!("Could not add host, no parent domain to resolve it."))
    }
}

fn show(filename: &str) -> Result<(), color_eyre::Report> {
    let mut hf = HostFile {
        filename: filename.to_string(),
        entries: None,
    };

    match hf.parse() {
        Ok(()) => {
            for item in hf.entries.unwrap_or_else(|| vec![]) {
                println!("{}", item); //item.unwrap_or_else(HostEntry::empty))
            }

            Ok(())
        }
        Err(x) => Err(eyre!("Failed to parse file {}", x)),
    }
}

fn verify(filename: &str) -> Result<(), color_eyre::Report> {
    let mut hf = HostFile {
        filename: filename.to_string(),
        entries: None,
    };

    match hf.parse() {
        Ok(()) => {
            println!(
                "Hostsfile is readable and contains {}{}{} entries.",
                color::Fg(color::Green),
                hf.entries.unwrap_or_else(Vec::new).len(),
                color::Fg(color::Reset),
            );
            Ok(())
        }
        Err(x) => Err(eyre!("Failed to parse file {}", x)),
    }
}

fn delete(filename: &str, entry: Option<&str>) -> Result<(), color_eyre::Report> {
    let mut hf = HostFile {
        filename: filename.to_string(),
        entries: None,
    };

    let is_ip = match entry.unwrap().parse::<IpAddr>() {
        Ok(_e) => true,
        Err(_e) => false,
    };

    match hf.parse() {
        Ok(()) => {
            if is_ip {
                hf.remove_ip(entry);
            } else {
                hf.remove_name(entry);
            }
            Ok(())
        }
        Err(x) => Err(eyre!("Failed to parse file {}", x)),
    }
}
