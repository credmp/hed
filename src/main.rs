use std::fs::File;
use std::io::{self, BufRead};
use std::net::IpAddr;
use std::path::Path;

fn main() {
    println!(
        "{:#?}",
        parse_line("10.10.10.111 forge.htb admin.forge.htb")
    );
    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines("./hosts") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            match line {
                Ok(ip) => {
                    println!("{}", ip);
                }
                _ => (),
            }
        }
    }
}

#[derive(Debug)]
pub struct HostEntry {
    ip: IpAddr,
    name: String,
    aliasses: Vec<String>,
}

fn parse_line(input: &str) -> Result<HostEntry, Box<dyn std::error::Error>> {
    Ok(HostEntry {
        ip: "10.10.10.111".parse()?,
        name: String::from("forge.htb"),
        aliasses: vec![String::from("admin.forge.htb")],
    })
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
}
