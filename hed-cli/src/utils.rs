use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Default, PartialEq, Debug)]
pub struct Modifications {
    pub updated_entries: i32,
    pub removed_entries: i32,
    pub added_entries: i32,
}

impl Modifications {
    pub fn new() -> Modifications {
        Modifications {
            updated_entries: 0,
            removed_entries: 0,
            added_entries: 0,
        }
    }

    pub(crate) fn merge(&mut self, m: Modifications) {
        self.updated_entries += m.updated_entries;
        self.removed_entries += m.removed_entries;
        self.added_entries += m.added_entries;
    }
}
