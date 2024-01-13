use std::{fmt, io::Write, net::IpAddr, str::FromStr, string::ParseError};

use regex::Regex;
use termion::color;

use crate::utils::Modifications;

#[derive(Debug, Clone)]
pub struct HostEntry {
    pub ip: Option<IpAddr>,
    pub name: Option<String>,
    pub aliasses: Option<Vec<String>>,
    pub comment: Option<String>,
}

impl HostEntry {
    pub fn empty() -> HostEntry {
        HostEntry {
            ip: None,
            name: None,
            aliasses: None,
            comment: None,
        }
    }

    pub fn color_print<W: Write>(&self, f: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        if self.ip.is_none() && self.comment.is_some() {
            writeln!(
                f,
                "{}# {}{}",
                color::Fg(color::LightBlue),
                self.comment.as_ref().unwrap(),
                color::Fg(color::Reset),
            )?;
        } else if self.ip.is_some() {
            write!(
                f,
                "{}{}\t{}{}\t{}{}",
                color::Fg(color::Cyan),
                self.ip.unwrap(),
                color::Fg(color::LightMagenta),
                self.name.as_ref().unwrap(),
                color::Fg(color::LightGreen),
                self.aliasses.as_ref().unwrap_or(&vec![]).join(" ")
            )?;
            if self.comment.is_some() {
                write!(
                    f,
                    "{}\t# {}",
                    color::Fg(color::LightBlue),
                    self.comment.as_ref().unwrap()
                )?;
            }
            writeln!(f, "{}", color::Fg(color::Reset))?;
        } else {
            writeln!(f)?;
        }
        Ok(())
    }

    /// Checks if the `name` of `HostEntry` can resolve the passed `hostname`.
    ///
    /// If `name` is `host.tld` and `hostname` is a subdomain, return true.
    /// If `name` is a subdomain `sub.host.tld` of `hostname`, return false.
    ///
    /// A subdomain is more specific, to rule out overlap, do not change it.
    /// TODO: allow reassigning of `name`
    pub fn can_resolve_host(&self, hostname: &str) -> bool {
        if self.name.is_some() {
            hostname.ends_with(format!(".{}", self.name.as_ref().unwrap().as_str()).as_str())
        } else {
            false
        }
    }

    /// Checks if the `hostname` can resolve the `name` of `HostEntry`. This
    /// is the inverse of `can_resolve_host`.
    ///
    /// If `hostname` is `host.tld` and `name` is a subdomain, return true.
    /// If `hostname` is a subdomain `sub.host.tld` of `name`, return false.
    ///
    pub fn can_hostname_resolve_domain(&self, hostname: &str) -> bool {
        if self.name.is_some() {
            self.name
                .as_ref()
                .unwrap()
                .as_str()
                .ends_with(format!(".{}", hostname).as_str())
        } else {
            false
        }
    }

    pub fn has_ip(&self, ip: &IpAddr) -> bool {
        if self.ip.is_some() {
            self.ip.as_ref().unwrap() == ip
        } else {
            false
        }
    }

    pub fn has_name(&self, hostname: &str) -> bool {
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

    /// a hostname is given that can be the parent domain
    /// make the hostname the `name` and move the current
    /// `name` to aliases    
    pub(crate) fn switch_name_with_alias(&mut self, hostname: &str) {
        if self.can_hostname_resolve_domain(hostname) {
            let t = self.name.as_ref().unwrap().clone();
            self.name = Some(String::from(hostname));
            self.add_alias(t.as_str());
        }
    }

    /// Indicate if the entire hostentry can be removed if the name is
    /// removed. Can only occur if the aliasses are empty.
    pub(crate) fn can_delete(&self, name: &str) -> bool {
        // name equals self.name and no aliasses
        (self.name.is_some() && name == self.name.as_ref().unwrap()) && self.aliasses.is_none()
    }

    /// Update the host entry by:
    ///
    /// - Removing an alias
    /// - Remove the name, chosing the shortest alias as the new name
    pub(crate) fn remove_hostname(&mut self, name: &str) -> (Modifications, HostEntry) {
        let mut mods = Modifications::new();

        // if it is the name that needs to be removed
        if let Some(n) = &self.name {
            if n == name {
                if let Some(aliasses) = &self.aliasses {
                    //                    let names = self.aliasses.as_ref().unwrap();
                    let shortest = aliasses.iter().fold(aliasses[0].to_owned(), |acc, item| {
                        if item.len() < acc.len() {
                            item.to_owned()
                        } else {
                            acc
                        }
                    });
                    let mut others: Vec<String> = vec![];
                    for x in aliasses {
                        if !shortest.eq(x) {
                            others.push(x.to_owned());
                        } else {
                            mods.updated_entries += 1;
                        }
                    }
                    return (
                        mods,
                        HostEntry {
                            ip: self.ip,
                            name: Some(shortest),
                            aliasses: Some(others),
                            comment: self.comment.clone(),
                        },
                    );
                } else {
                    // name is the same, and no aliasses... should not happen
                    return (mods, HostEntry::empty());
                }
            } else if let Some(aliasses) = &self.aliasses {
                //                let names = aliasses.as_ref().unwrap();
                let mut others: Vec<String> = vec![];
                for x in aliasses {
                    if name != x {
                        others.push(x.to_owned());
                    } else {
                        mods.updated_entries += 1;
                    }
                }
                return (
                    mods,
                    HostEntry {
                        ip: self.ip,
                        name: self.name.clone(),
                        aliasses: Some(others),
                        comment: self.comment.clone(),
                    },
                );
            }
        }
        (mods, self.clone())
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

impl FromStr for HostEntry {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let comment = Regex::new(r"^#\s*(?P<c>.+)\s*$").unwrap();
        let entry =
            Regex::new(r"^(?P<ip>.+?)\s+(?P<name>.+?)(\s+(?P<aliasses>[^#]+))?(#\s*(?P<c>.*))?$")
                .unwrap();
        if comment.is_match(s) {
            Ok(comment
                .captures(s)
                .map(|cap| HostEntry {
                    ip: None,
                    name: None,
                    aliasses: None,
                    comment: cap.name("c").map(|t| String::from(t.as_str().trim())),
                })
                .unwrap())
        } else if entry.is_match(s) {
            let caps = entry.captures(s).unwrap();
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
            let comment = caps.name("c").map(|t| String::from(t.as_str().trim()));
            Ok(HostEntry {
                ip,
                name,
                aliasses: alias_vec,
                comment,
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
}

impl fmt::Display for HostEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.ip.is_none() && self.comment.is_some() {
            write!(f, "# {}", self.comment.as_ref().unwrap(),)
        } else if self.ip.is_some() {
            write!(f, "{}\t{}", self.ip.unwrap(), self.name.as_ref().unwrap(),)?;
            if self.aliasses.is_some() {
                write!(
                    f,
                    "\t{}",
                    self.aliasses.as_ref().unwrap_or(&vec![]).join(" ")
                )
                .expect("Initial write succeeded, this should too");
            }
            if self.comment.is_some() {
                write!(f, "\t# {}", self.comment.as_ref().unwrap())
                    .expect("Initial write succeeded, this should too");
            }
            Ok(())
        } else {
            write!(f, "")
        }
    }
}

#[cfg(test)]
mod test {

    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use crate::hostentry::HostEntry;

    #[test]
    fn test_ip_addr() {
        let localhost_v4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let localhost_v6 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));

        assert_eq!("127.0.0.1".parse(), Ok(localhost_v4));
        assert_eq!("::1".parse(), Ok(localhost_v6));
    }

    #[test]
    fn test_remove_hostname() {
        let ip = "127.0.0.1".parse().expect("Should parse localhost");
        let mut he = HostEntry {
            ip: Some(ip),
            name: Some(String::from("arjenwiersma.nl")),
            aliasses: None,
            comment: None,
        };

        assert!(he.can_delete("arjenwiersma.nl"));
        let res = he.remove_hostname("arjenwiersma.nl");
        assert_eq!(res.1, HostEntry::empty());

        he.aliasses = Some(vec![
            String::from("demo.arjenwiersma.nl"),
            String::from("d.arjenwiersma.nl"),
        ]);

        assert!(!he.can_delete("arjenwiersma.nl"));
        assert_eq!(
            he.remove_hostname("arjenwiersma.nl").1,
            HostEntry {
                ip: Some(ip),
                name: Some(String::from("d.arjenwiersma.nl")),
                aliasses: Some(vec![String::from("demo.arjenwiersma.nl")]),
                comment: None,
            }
        );

        assert_eq!(
            he.remove_hostname("demo.arjenwiersma.nl").1,
            HostEntry {
                ip: Some(ip),
                name: Some(String::from("arjenwiersma.nl")),
                aliasses: Some(vec![String::from("d.arjenwiersma.nl")]),
                comment: None,
            }
        );
    }

    #[test]
    fn test_add_alias() {
        let ip = "127.0.0.1".parse().expect("Should parse localhost");
        let mut he = HostEntry {
            ip: Some(ip),
            name: Some(String::from("arjenwiersma.nl")),
            aliasses: None,
            comment: None,
        };

        he.add_alias("test.nl");
        assert_eq!(he.aliasses.unwrap().len(), 1);
    }
    #[test]
    fn test_display() {
        let ip = "127.0.0.1".parse().expect("Should parse localhost");
        let mut he = HostEntry {
            ip: Some(ip),
            name: Some(String::from("arjenwiersma.nl")),
            aliasses: None,
            comment: None,
        };
        assert_eq!("127.0.0.1\tarjenwiersma.nl", format!("{}", he));
        he.comment = Some(String::from("a comment"));
        assert_eq!("127.0.0.1\tarjenwiersma.nl\t# a comment", format!("{}", he));
        he.aliasses = Some(vec![String::from("host1"), String::from("host2")]);
        assert_eq!(
            "127.0.0.1\tarjenwiersma.nl\thost1 host2\t# a comment",
            format!("{}", he)
        );
        he.comment = None;
        assert_eq!("127.0.0.1\tarjenwiersma.nl\thost1 host2", format!("{}", he));
    }

    #[test]
    fn test_has_ip() {
        let ip = "127.0.0.1".parse().expect("Should parse localhost");
        let he = HostEntry {
            ip: Some(ip),
            name: None,
            aliasses: None,
            comment: None,
        };

        let ip2 = "127.0.0.1".parse().expect("Should parse localhost");
        assert!(he.has_ip(&ip2));
        let ip3 = "127.0.1.1".parse().expect("Should parse localhost");
        assert!(!he.has_ip(&ip3));
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

    #[test]
    fn test_can_resolve() {
        let he = HostEntry {
            ip: None,
            name: Some(String::from("wiersma.nl")),
            aliasses: None,
            comment: None,
        };

        assert!(he.can_resolve_host("arjen.wiersma.nl"));
        assert!(!he.can_resolve_host("bwiersma.nl"));
        assert!(!he.can_resolve_host("piet.nl"));
    }

    #[test]
    fn test_hostname_can_resolve() {
        let he = HostEntry {
            ip: None,
            name: Some(String::from("arjen.wiersma.nl")),
            aliasses: None,
            comment: None,
        };

        assert!(!he.can_hostname_resolve_domain("iersma.nl"));
        assert!(he.can_hostname_resolve_domain("wiersma.nl"));
        assert!(!he.can_hostname_resolve_domain("piet.nl"));
    }

    #[test]
    fn test_switch_name() {
        let mut he = HostEntry {
            ip: None,
            name: Some(String::from("arjen.wiersma.nl")),
            aliasses: None,
            comment: None,
        };

        he.switch_name_with_alias("wiersma.nl");

        assert_eq!(he.name.unwrap(), "wiersma.nl");
        assert_eq!(he.aliasses.unwrap().get(0).unwrap(), "arjen.wiersma.nl");
    }

    #[test]
    fn test_from_string_entry() {
        let entry: HostEntry = "1.1.1.1    arjenwiersma.nl".parse().unwrap();

        assert_eq!("arjenwiersma.nl", entry.name.unwrap());
        assert_eq!("1.1.1.1".parse::<IpAddr>().unwrap(), entry.ip.unwrap());
    }

    #[test]
    fn test_from_string_empty_entry() {
        let entry: HostEntry = "".parse().unwrap();

        assert_eq!(None, entry.name);
        assert_eq!(None, entry.ip);
    }

    #[test]
    fn test_from_string_entry_with_alias() {
        let entry: HostEntry = "1.1.1.1    arjenwiersma.nl alias1 alias2".parse().unwrap();

        assert_eq!("arjenwiersma.nl", entry.name.unwrap());
        assert_eq!("1.1.1.1".parse::<IpAddr>().unwrap(), entry.ip.unwrap());
        assert_eq!(entry.aliasses.unwrap(), vec!["alias1", "alias2"]);
    }

    #[test]
    fn test_from_string_entry_with_alias_and_comment() {
        let entry: HostEntry = "1.1.1.1    arjenwiersma.nl alias1 alias2 # testing"
            .parse()
            .unwrap();

        assert_eq!("arjenwiersma.nl", entry.name.unwrap());
        assert_eq!("1.1.1.1".parse::<IpAddr>().unwrap(), entry.ip.unwrap());
        assert_eq!(entry.aliasses.unwrap(), vec!["alias1", "alias2"]);
        assert_eq!(entry.comment.unwrap(), "testing");
    }

    #[test]
    fn test_from_string_comment() {
        let entry: HostEntry = "# from string".parse().unwrap();

        assert_eq!("from string", entry.comment.unwrap());
    }
}
