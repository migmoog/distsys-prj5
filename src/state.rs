use bootstrap::NodeId;
use messaging::Message;
use tokio::{io, net::TcpStream};

use crate::setup::{
    hostsfile::Objects,
    socketry::{attempt_op, host},
};

pub mod bootstrap;
pub mod messaging;

pub struct Peer {
    obj: Objects,
    to_boot: TcpStream,
    nid: NodeId,
}
impl Peer {
    pub async fn new(obj: Objects, to_bootstrap: String) -> io::Result<Self> {
        let to_boot = attempt_op(TcpStream::connect, &to_bootstrap).await;
        let hn = host();
        let nid = hn
            .strip_prefix("n")
            .and_then(|v| v.parse::<NodeId>().ok())
            .unwrap();
        Ok(Self { obj, to_boot, nid })
    }

    pub async fn join(&mut self) -> io::Result<()> {
        let msg = Message::Join(self.nid);
        msg.send(&mut self.to_boot).await?;
        Ok(())
    }
}
