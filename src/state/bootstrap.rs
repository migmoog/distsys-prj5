use indexmap::IndexSet;
use tokio::{
    io,
    net::{TcpListener, UdpSocket},
    sync::mpsc::UnboundedReceiver,
};

use crate::setup::socketry::{attempt_op, bootstrap_comms, host, NodeCaster};

use super::messaging::Message;

pub type NodeId = u64;
pub struct Ring {
    nodes: IndexSet<NodeId>,
    recv: UnboundedReceiver<Message>,
    pusher: NodeCaster,
}

impl Ring {
    pub async fn new() -> tokio::io::Result<Self> {
        let hn = host();
        let listener = attempt_op(TcpListener::bind, &hn, 6969).await;
        let recv = bootstrap_comms(listener);
        let pusher = NodeCaster::new().await;

        Ok(Self {
            nodes: IndexSet::new(),
            recv,
            pusher,
        })
    }

    pub fn poll(&mut self) -> Option<Message> {
        self.recv.try_recv().ok()
    }

    pub async fn respond_to_join(&mut self, nid: NodeId) -> io::Result<()> {
        self.nodes.insert(nid);
        self.nodes.sort(); // Ring ordering

        let len = self.nodes.len();
        if len <= 1 {
            return Ok(()); // Avoids unneccesary prints 🤷‍♀️
        }
        let idx = self.nodes.get_index_of(&nid).unwrap();

        let pred = self
            .nodes
            .get_index(if idx == 0 { len - 1 } else { idx - 1 })
            .unwrap();
        let succ = self.nodes.get_index((idx + 1) % len).unwrap();

        for (target, msg) in [
            (*pred, Message::NewSuccessor(nid)),
            (nid, Message::NewPredecessor(*pred)),
            (*succ, Message::NewPredecessor(nid)),
            (nid, Message::NewSuccessor(*succ)),
        ] {
            self.pusher.tell_node(msg, target).await?;
        }

        Ok(())
    }
}
