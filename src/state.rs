use bootstrap::NodeId;
use messaging::Message;
use tokio::{io, net::TcpStream};

use crate::setup::{
    hostsfile::Objects,
    socketry::{attempt_op, host, NodeCaster},
};

pub mod bootstrap;
pub mod messaging;

pub struct Peer {
    obj: Objects,
    to_boot: TcpStream,
    nid: NodeId,

    puller: NodeCaster,
    predecessor: NodeId,
    successor: NodeId,
}
impl Peer {
    pub async fn new(obj: Objects, to_bootstrap: String) -> io::Result<Self> {
        let hn = host();
        let to_boot = attempt_op(TcpStream::connect, &to_bootstrap, 6969).await;
        let puller = NodeCaster::new().await;
        let nid = hn
            .strip_prefix("n")
            .and_then(|v| v.parse::<NodeId>().ok())
            .unwrap();
        Ok(Self {
            obj,
            to_boot,
            nid,

            puller,
            predecessor: nid,
            successor: nid,
        })
    }

    pub async fn join(&mut self) -> io::Result<()> {
        let msg = Message::Join(self.nid);
        msg.send(&mut self.to_boot).await?;
        Ok(())
    }

    pub async fn hear(&mut self) -> io::Result<()> {
        let msg = self.puller.hear().await?;
        match msg {
            Message::NewSuccessor(nid) => {
                self.successor = nid;
                self.ring_print();
            }
            Message::NewPredecessor(nid) => {
                self.predecessor = nid;
                self.ring_print();
            }
            _ => {}
        }
        Ok(())
    }

    fn ring_print(&self) {
        eprintln!(
            "Predecessor: n{}, Successor: n{}",
            self.predecessor, self.successor
        );
    }
}
