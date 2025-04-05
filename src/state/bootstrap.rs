use indexmap::IndexSet;
use tokio::{net::TcpListener, sync::mpsc::UnboundedReceiver};

use crate::setup::socketry::{attempt_op, bootstrap_comms};

use super::messaging::Message;

pub type NodeId = u64;
pub struct Ring {
    nodes: IndexSet<NodeId>,
    recv: UnboundedReceiver<Message>,
}

impl Ring {
    pub async fn new() -> tokio::io::Result<Self> {
        let listener = attempt_op(
            TcpListener::bind,
            &hostname::get().unwrap().into_string().unwrap(),
        )
        .await;
        let recv = bootstrap_comms(listener);

        Ok(Self {
            nodes: IndexSet::new(),
            recv,
        })
    }

    pub fn poll(&mut self) -> Option<Message> {
        self.recv.try_recv().ok()
    }

    pub fn respond_to_join(&mut self, nid: NodeId) {
        self.nodes.insert(nid);
        println!("Added {nid}");
    }
}
