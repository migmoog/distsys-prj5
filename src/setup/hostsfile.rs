use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};

pub type ClientId = u64;
pub type ObjectId = u64;

pub struct Objects {
    table: HashMap<ClientId, HashSet<ObjectId>>,
}
impl Objects {
    pub fn load(path: PathBuf) -> std::io::Result<Self> {
        let source = File::open(path)?;
        let reader = BufReader::new(&source);

        let mut table = HashMap::new();
        for line in reader.lines().flatten() {
            let Some((Ok(cid), Ok(oid))) = line
                .split_once("::")
                .and_then(|(c, o)| Some((c.parse::<ClientId>(), o.parse::<ObjectId>())))
            else {
                continue;
            };

            table
                .entry(cid)
                .and_modify(|objs: &mut HashSet<ObjectId>| {
                    objs.insert(oid);
                })
                .or_insert(HashSet::from([oid]));
        }

        Ok(Objects { table })
    }
}

/// Helper to keep track of whos who
#[derive(Debug)]
pub struct PeerList {}
impl PeerList {}
