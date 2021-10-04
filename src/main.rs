use clap::{App, Arg};
use color_eyre::eyre::eyre;
pub use color_eyre::eyre::Result;
use faccess::PathExt;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead, Error, ErrorKind};
use std::net::IpAddr;
use std::path::Path;
use std::{fmt, fs};

use regex::Regex;

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = App::new("hed")
        .version("0.0.1-alpha")
        .author("Arjen Wiersma <arjen@wiersma.org")
        .about("Host EDitor")
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

fn get_hostentries(filename: &str) -> Result<Vec<Option<HostEntry>>> {
    match parse_file(filename) {
        Ok(x) => Ok(x),
        Err(x) => Err(eyre!("{}", x)),
    }
}

fn write_file(filename: &str, hosts: &mut Vec<Option<HostEntry>>) -> Result<()> {
    println!("Writing to file {}", filename);

    let path = Path::new(filename);
    if !path.writable() {
        println!("Not writable {}, escalating", filename);
        if let Err(e) = sudo::escalate_if_needed() {
            return Err(eyre!("Failed to run as root! {}", e));
        }
    }

    if let Err(e) = fs::copy(filename, format!("{}.bak", filename)) {
        return Err(eyre!(
            "Failed to write backup file, refusing to overwrite original ({})",
            e
        ));
    }

    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    for item in hosts.iter_mut().flatten() {
        if let Err(why) = writeln!(file, "{}", item) {
            panic!("couldn't write to {}: {}", display, why)
        }
    }
    Ok(())
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

    let hosts = &mut get_hostentries(filename)?;

    // if IP address is given, find a matching hostentry to add a alias
    //    no ip? add new entry
    // if only a name is given, find a HostEntry already serving a tld
    //    if none are found, err
    if let Some(ip_a) = ip_address {
        for item in hosts.iter_mut().flatten() {
            let i = item;

            if i.has_ip(&ip_a) {
                if !i.has_name(hostname.unwrap()) {
                    i.add_alias(hostname.unwrap());
                    return write_file(filename, hosts);
                }
            }
        }

        hosts.push(Some(HostEntry {
            ip: ip_address,
            name: Some(hostname.unwrap().to_string()),
            comment: None,
            aliasses: None,
        }));
        write_file(filename, hosts)
    } else {
        for item in hosts.iter_mut().flatten() {
            let i = item;

            if i.can_resolve_host(hostname.unwrap()) && !i.has_name(hostname.unwrap()) {
                i.add_alias(hostname.unwrap());
                return write_file(filename, hosts);
            }
        }

        Err(eyre!("Could not add host, no parent domain to resolve it."))
    }
}

fn show(filename: &str) -> Result<(), color_eyre::Report> {
    let hosts = get_hostentries(filename)?;

    for item in hosts {
        println!("{}", item.unwrap_or_else(HostEntry::empty))
    }
    Ok(())
}

fn verify(filename: &str) -> Result<(), color_eyre::Report> {
    match parse_file(filename) {
        Ok(x) => {
            println!("Hostsfile is readable and contains {} entries", x.len());
            Ok(())
        }
        Err(x) => Err(eyre!("Failed to parse file {}", x)),
    }
}

#[derive(Debug)]
pub struct HostEntry {
    ip: Option<IpAddr>,
    name: Option<String>,
    aliasses: Option<Vec<String>>,
    comment: Option<String>,
}

impl HostEntry {
    fn empty() -> HostEntry {
        HostEntry {
            ip: None,
            name: None,
            aliasses: None,
            comment: None,
        }
    }

    /// Checks if the `name` of `HostEntry` can result the passed `hostname`.
    ///
    /// If `name` is `host.tld` and `hostname` is a subdomain, return true.
    /// If `name` is a subdomain `sub.host.tld` of `hostname`, return false.
    ///
    /// A subdomain is more specific, to rule out overlap, do not change it.
    /// TODO: allow reassigning of `name`
    fn can_resolve_host(&self, hostname: &str) -> bool {
        if self.name.is_some() {
            hostname.ends_with(self.name.as_ref().unwrap().as_str())
        } else {
            false
        }
    }

    fn has_ip(&self, ip: &IpAddr) -> bool {
        if self.ip.is_some() {
            self.ip.as_ref().unwrap() == ip
        } else {
            false
        }
    }

    fn has_name(&self, hostname: &str) -> bool {
        if let Some(x) = &self.name {
            if x.eq(hostname) {
                return true;
            }
        }
        match &self.aliasses {
            Some(y) => {
                for z in y {
                    if z == hostname {
                        return true;
                    }
                }
            }
            _ => {
                return false;
            }
        }
        false
    }

    pub(crate) fn add_alias(&mut self, hostname: &str) {
        if self.aliasses.is_some() {
            let mut alias: Vec<String> = self.aliasses.as_ref().unwrap().clone();
            alias.push(hostname.to_string());
            self.aliasses = Some(alias);
        } else {
            self.aliasses = Some(vec![hostname.to_string()]);
        }
    }
}

impl PartialEq for HostEntry {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip
            && self.name == other.name
            && self.aliasses == other.aliasses
            && self.comment == other.comment
    }
}

// impl PartialOrd for HostEntry {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         todo!()
//     }
// }

impl fmt::Display for HostEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.ip == None && self.comment != None {
            write!(f, "# {}", self.comment.as_ref().unwrap())
        } else if self.ip != None {
            write!(
                f,
                "{}\t{}\t{}",
                self.ip.unwrap(),
                self.name.as_ref().unwrap(),
                self.aliasses.as_ref().unwrap_or(&vec![]).join(" ")
            )
        } else {
            write!(f, "")
        }
    }
}

fn parse_file(filename: &str) -> Result<Vec<Option<HostEntry>>, Box<dyn std::error::Error>> {
    match read_lines(filename) {
        Ok(lines) => Ok(lines
            .map(
                |l| match parse_line(l.unwrap_or_else(|_e| "".to_string()).as_str()) {
                    Ok(s) => Some(s),
                    _ => None,
                },
            )
            .collect()),
        Err(e) => Err(Box::new(Error::new(
            ErrorKind::Other,
            format!("Failed to parse file {}", e),
        ))),
    }
}

fn parse_line(input: &str) -> Result<HostEntry, Box<dyn std::error::Error>> {
    let comment = Regex::new(r"^#(?P<c>.+)\s*$").unwrap();
    let entry = Regex::new(r"^(?P<ip>.+?)\s+(?P<name>.+?)(\s+(?P<aliasses>.+))?$").unwrap();
    if comment.is_match(input) {
        Ok(comment
            .captures(input)
            .map(|cap| HostEntry {
                ip: None,
                name: None,
                aliasses: None,
                comment: cap.name("c").map(|t| String::from(t.as_str())),
            })
            .unwrap())
    } else if entry.is_match(input) {
        let caps = entry.captures(input).unwrap();
        let ip_str = caps.name("ip").map(|t| t.as_str()).unwrap();

        let ip: Option<IpAddr> = match ip_str.parse() {
            Ok(x) => Some(x),
            _ => None,
        };

        let name = caps.name("name").map(|t| String::from(t.as_str().trim()));
        let alias = caps
            .name("aliasses")
            .map(|t| String::from(t.as_str().trim()));
        let alias_vec: Option<Vec<String>> = alias.map(|x| {
            x.split_whitespace()
                .map(String::from)
                .collect::<Vec<String>>()
        });
        Ok(HostEntry {
            ip,
            name,
            aliasses: alias_vec,
            comment: None,
        })
    } else {
        Err(Box::new(Error::new(
            ErrorKind::Other,
            "Failed to read host string",
        )))
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[cfg(test)]
mod test {

    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

    #[test]
    fn test_ip_addr() {
        let localhost_v4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let localhost_v6 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));

        assert_eq!("127.0.0.1".parse(), Ok(localhost_v4));
        assert_eq!("::1".parse(), Ok(localhost_v6));
    }

    #[test]
    fn test_hostentry_has_name() {
        let he = HostEntry {
            ip: None,
            name: Some(String::from("arjen.wiersma.nl")),
            aliasses: None,
            comment: None,
        };

        assert!(he.has_name("arjen.wiersma.nl"));
        assert!(!he.has_name("arjen2.wiersma.nl"));
        assert!(!he.has_name("wiersma.nl"));

        let ahe = HostEntry {
            ip: None,
            name: Some(String::from("jelle.wiersma.nl")),
            aliasses: Some(vec![
                String::from("arjen.wiersma.nl"),
                String::from("rebecca.wiersma.nl"),
            ]),
            comment: None,
        };
        assert!(ahe.has_name("arjen.wiersma.nl"));
        assert!(ahe.has_name("rebecca.wiersma.nl"));
        assert!(!ahe.has_name("arjen2.wiersma.nl"));
        assert!(!ahe.has_name("wiersma.nl"));
        assert!(ahe.has_name("jelle.wiersma.nl"));
    }
}
