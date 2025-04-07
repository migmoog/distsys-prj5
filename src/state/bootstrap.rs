use std::sync::Arc;

use indexmap::IndexSet;
use tokio::{
    io,
    net::{TcpListener, UdpSocket},
    sync::mpsc::UnboundedReceiver,
};

use crate::setup::{
    hostsfile::{ClientId, ObjectId},
    socketry::{attempt_op, bootstrap_comms, host, NodeCaster},
};

use super::messaging::{Message, Operation, RequestId};

pub type NodeId = u64;
pub struct Ring {
    nodes: IndexSet<NodeId>,
    recv: UnboundedReceiver<Message>,
    pusher: NodeCaster,
    client_hub: Arc<UdpSocket>, // Socket for any clients to send datagrams to
}

impl Ring {
    pub async fn new() -> tokio::io::Result<Self> {
        let hn = host();
        let listener = attempt_op(TcpListener::bind, &hn, 6969).await;
        let (send, recv) = bootstrap_comms(listener);
        let pusher = NodeCaster::new().await;

        let client_hub = Arc::new(attempt_op(UdpSocket::bind, &hn, 6971).await);
        let ch = Arc::clone(&client_hub);
        // poll to receive requests from client
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                if let Ok((br, addr)) = ch.recv_from(&mut buffer).await {
                    ch.connect(addr).await.expect("Connects back to client");
                    let msg = bincode::deserialize(&buffer[..br]).unwrap();
                    send.send(msg).expect("Sends gud");
                }
            }
        });

        Ok(Self {
            nodes: IndexSet::new(),
            recv,
            pusher,
            client_hub,
        })
    }

    pub fn poll(&mut self) -> Option<Message> {
        self.recv.try_recv().ok()
    }

    pub async fn respond_to_client(&mut self, msg: Message) -> io::Result<()> {
        let encoded = bincode::serialize(&msg).unwrap();
        self.client_hub.send(&encoded).await?;
        Ok(())
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

    /// Sends a client's request to n1 so it can be passed around the rings
    pub async fn drop_request(&mut self, msg: Message) -> io::Result<()> {
        assert!(matches!(msg, Message::REQUEST(_, _, _, _)));
        self.pusher.tell_node(msg, 1).await?;
        Ok(())
    }
}

pub struct Client {
    id: ClientId,
    reqs_made: RequestId,
    channel: UdpSocket,
}
impl Client {
    const STORE_EXAMPLE: ObjectId = 115; // should be stored by n126
    const RETRIEVE_EXAMPLE: ObjectId = 45; // stored in n50
    const NOT_FOUND_EXAMPLE: ObjectId = 60; // does not exist, n66 will return a not found.
    pub async fn new(to_bootstrap: String) -> io::Result<Self> {
        let channel = attempt_op(UdpSocket::bind, &host(), 6971).await;
        channel.connect(format!("{to_bootstrap}:6971")).await?;
        Ok(Self {
            id: 1,
            reqs_made: 0,
            channel,
        })
    }

    pub async fn send_req(&mut self, tno: u64) -> io::Result<()> {
        self.reqs_made += 1;
        let (oid, op) = match tno {
            3 => (Self::STORE_EXAMPLE, Operation::STORE),
            4 => (Self::RETRIEVE_EXAMPLE, Operation::RETRIEVE),
            5 => (Self::NOT_FOUND_EXAMPLE, Operation::RETRIEVE),
            _ => unreachable!("Should only run for testcases [3, 5]"),
        };
        let msg = Message::REQUEST(self.reqs_made, self.id, oid, op);
        let encoded = bincode::serialize(&msg).unwrap();
        self.channel.send(&encoded).await?;
        Ok(())
    }

    pub async fn wait_for_res(&mut self) -> io::Result<()> {
        let mut buffer = [0; 1024];
        let br = self.channel.recv(&mut buffer).await?;
        match bincode::deserialize::<Message>(&buffer[..br]).unwrap() {
            Message::OBJ_RETRIEVED(o, _, _) => {
                eprintln!("RETRIEVED: {o}");
            }
            Message::OBJ_STORED(o, _, _) => {
                eprintln!("STORED: {o}");
            }
            Message::NOT_FOUND(o) => {
                eprintln!("NOT FOUND: {o}");
            }
            _ => {}
        }

        Ok(())
    }
}
