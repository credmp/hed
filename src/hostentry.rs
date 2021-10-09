use std::{fmt, net::IpAddr};

use termion::color;

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

    pub fn color_print(&self) {
        if self.ip == None && self.comment != None {
            println!(
                "{}# {}{}",
                color::Fg(color::LightBlue),
                self.comment.as_ref().unwrap(),
                color::Fg(color::Reset),
            );
        } else if self.ip != None {
            println!(
                "{}{}\t{}{}\t{}{}{}",
                color::Fg(color::Cyan),
                self.ip.unwrap(),
                color::Fg(color::LightMagenta),
                self.name.as_ref().unwrap(),
                color::Fg(color::LightGreen),
                self.aliasses.as_ref().unwrap_or(&vec![]).join(" "),
                color::Fg(color::Reset),
            );
        } else {
            println!();
        }
    }

    /// Checks if the `name` of `HostEntry` can result the passed `hostname`.
    ///
    /// If `name` is `host.tld` and `hostname` is a subdomain, return true.
    /// If `name` is a subdomain `sub.host.tld` of `hostname`, return false.
    ///
    /// A subdomain is more specific, to rule out overlap, do not change it.
    /// TODO: allow reassigning of `name`
    pub fn can_resolve_host(&self, hostname: &str) -> bool {
        if self.name.is_some() {
            hostname.ends_with(self.name.as_ref().unwrap().as_str())
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

    /// Indicate if the entire hostname can be removed if the name is
    /// removed. Can only occur if the aliasses are empty.
    pub(crate) fn can_delete(&self, name: &str) -> bool {
        // name equals self.name and no aliasses
        (self.name.is_some() && name == self.name.as_ref().unwrap()) && self.aliasses.is_none()
    }

    /// Update the host entry by:
    ///
    /// - Removing an alias
    /// - Remove the name, chosing the shortest alias as the new name
    pub(crate) fn remove_hostname(&mut self, name: &str) -> HostEntry {
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
                        }
                    }
                    return HostEntry {
                        ip: self.ip,
                        name: Some(shortest),
                        aliasses: Some(others),
                        comment: None,
                    };
                } else {
                    // name is the same, and no aliasses... should not happen
                    return HostEntry::empty();
                }
            } else if let Some(aliasses) = &self.aliasses {
                //                let names = aliasses.as_ref().unwrap();
                let mut others: Vec<String> = vec![];
                for x in aliasses {
                    if name != x {
                        others.push(x.to_owned());
                    }
                }
                return HostEntry {
                    ip: self.ip,
                    name: self.name.clone(),
                    aliasses: Some(others),
                    comment: None,
                };
            }
        }
        self.clone()
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
            write!(f, "# {}", self.comment.as_ref().unwrap(),)
        } else if self.ip != None {
            write!(
                f,
                "{}\t{}\t{}",
                self.ip.unwrap(),
                self.name.as_ref().unwrap(),
                self.aliasses.as_ref().unwrap_or(&vec![]).join(" "),
            )
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
