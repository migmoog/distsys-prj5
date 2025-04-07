use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    path::PathBuf,
};

pub type ClientId = u64;
pub type ObjectId = u64;

pub struct Objects(File);
impl Objects {
    pub fn load(path: PathBuf) -> std::io::Result<Self> {
        let source = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .open(path)?;
        Ok(Self(source))
    }

    pub fn print_objects(&mut self) -> std::io::Result<()> {
        self.0.seek(SeekFrom::Start(0))?;
        for line in BufReader::new(&self.0).lines().filter_map(Result::ok) {
            eprintln!("{line}");
        }
        Ok(())
    }

    pub fn store_object(&mut self, cid: ClientId, oid: ObjectId) -> std::io::Result<()> {
        self.0.write_fmt(format_args!("{cid}::{oid}\n"))?;
        Ok(())
    }

    pub fn retrieve_object(&mut self, cid: ClientId, oid: ObjectId) -> bool {
        let reader = BufReader::new(&self.0);
        let entry = format!("{cid}::{oid}");
        reader
            .lines()
            .filter_map(Result::ok)
            .any(|line| line == entry)
    }
}
