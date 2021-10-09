use faccess::PathExt;
use std::io::Write;
use std::{
    fs::{self, File},
    io::{Error, ErrorKind},
    net::IpAddr,
    path::Path,
};
use termion::color;

use regex::Regex;

use crate::hostentry::HostEntry;
use crate::utils::read_lines;

#[derive(Debug)]
pub struct HostFile {
    pub filename: String,
    pub entries: Option<Vec<HostEntry>>,
}

impl HostFile {
    pub fn add_host_entry(&mut self, entry: HostEntry) {
        if self.entries.is_some() {
            let mut e: Vec<HostEntry> = self.entries.as_ref().unwrap().clone();
            e.push(entry);
            self.entries = Some(e);
        } else {
            self.entries = Some(vec![entry]);
        }
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Writing to file {}", self.filename);

        let path = Path::new(&self.filename);
        if !path.writable() {
            println!("Not writable {}, escalating", self.filename);
            if let Err(e) = sudo::escalate_if_needed() {
                return Err(Box::new(Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse file {}", e),
                )));
            }
        }

        if let Err(e) = fs::copy(&self.filename, format!("{}.bak", self.filename)) {
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                format!(
                    "Failed to write backup file, refusing to overwrite original ({})",
                    e
                ),
            )));
        }

        let display = path.display();

        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        for item in self.entries.as_ref().unwrap() {
            if let Err(why) = writeln!(file, "{}", item) {
                panic!("couldn't write to {}: {}", display, why)
            }
        }
        Ok(())
    }

    pub fn parse(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Reading file {}", self.filename);
        match read_lines(&self.filename) {
            Ok(lines) => {
                self.entries = lines
                    .map(|l| {
                        match HostFile::parse_line(l.unwrap_or_else(|_e| "".to_string()).as_str()) {
                            Ok(s) => Some(s),
                            _ => None,
                        }
                    })
                    .collect();
                Ok(())
            }
            Err(e) => Err(Box::new(Error::new(ErrorKind::Other, e))),
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
            Ok(HostEntry {
                ip: None,
                name: None,
                aliasses: None,
                comment: None,
            })
        }
    }

    pub(crate) fn remove_ip(&mut self, entry: Option<&str>) {
        let ip: IpAddr = match entry.unwrap().parse() {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Invalid IP address given: {}", e);
                return;
            }
        };

        if self.entries.is_some() {
            let en = self.entries.as_ref().unwrap().clone();
            let c = en.len();
            self.entries = Some(
                en.into_iter()
                    .filter(|he| {
                        if he.ip.is_some() {
                            he.ip.unwrap() != ip
                        } else {
                            true
                        }
                    })
                    .collect::<Vec<_>>(),
            );
            println!(
                "Removed {}{}{} entries",
                color::Fg(color::Green),
                c - self.entries.as_ref().unwrap().len(),
                color::Fg(color::Reset)
            );
        } else {
            eprintln!("No entries to delete");
        }
    }

    pub(crate) fn remove_name(&mut self, name: Option<&str>) {
        // if the name is the `name` and no aliasses, remove the entry

        if self.entries.is_some() && name.is_some() {
            let en = self.entries.as_ref().unwrap().clone();
            let c = en.len();

            if let Some(n) = name {
                // // filter with 'can delete'.
                // self.entries = Some(
                //     en.into_iter()
                //         .filter(|he| !he.can_delete(n))
                //         .collect::<Vec<_>>(),
                // );

                // if the name is the `name`, find the shortest alias to take its place
                let mut updated: Vec<HostEntry> = vec![];
                for mut entry in self.entries.as_ref().unwrap().clone() {
                    if !entry.can_delete(n) {
                        updated.push(entry.remove_hostname(n));
                    }
                }

                self.entries = Some(updated);
            }
            println!(
                "Removed {}{}{} entries",
                color::Fg(color::Green),
                c - self.entries.as_ref().unwrap().len(),
                color::Fg(color::Reset)
            );
        } else {
            eprintln!("No entries to delete");
        }

        // else the entry needs to be update, if it is there
    }
}
