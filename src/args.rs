use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Project5 {
    /// The hostname of the bootstrap server.
    #[arg(short = 'b')]
    pub bootstrap: Option<String>,

    /// The number of seconds to wait before joining after startup.
    #[arg(short = 'd')]
    pub delay: Option<u64>,

    /// This argument only applies to client for testcases 3, 4, and 5.
    #[arg(short = 't')]
    pub testcase: Option<u64>,

    /// Path to a file containing the object store of the peer.
    #[arg(short = 'o')]
    pub object: Option<PathBuf>,
}
impl Project5 {
    pub fn is_bootstrap(&self) -> bool {
        self.bootstrap.is_none()
    }
}
