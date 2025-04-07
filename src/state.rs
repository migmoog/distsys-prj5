use bootstrap::NodeId;
use messaging::{Message, Operation, RequestId};
use tokio::{io, net::TcpStream};

use crate::setup::{
    hostsfile::{ObjectId, Objects},
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
        self.ring_print();
        let msg = Message::Join(self.nid);
        msg.send(&mut self.to_boot).await?;
        Ok(())
    }

    pub async fn hear(&mut self) -> io::Result<()> {
        let msg = self.puller.hear().await?;
        match &msg {
            Message::NewSuccessor(nid) => {
                self.successor = *nid;
                self.ring_print();
            }
            Message::NewPredecessor(nid) => {
                self.predecessor = *nid;
                self.ring_print();
            }
            Message::REQUEST(_rid, _cid, oid, _op) => {
                if self.owns_object(*oid) {
                    self.process_request(msg).await?;
                } else {
                    self.propagate_request(msg).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Processes a request and sends a message back to the bootstrap
    async fn process_request(&mut self, msg: Message) -> io::Result<()> {
        let Message::REQUEST(_rid, cid, oid, op) = msg else {
            return Ok(());
        };

        match op {
            Operation::STORE => {
                self.obj.store_object(cid, oid)?;
                self.obj.print_objects()?;

                Message::OBJ_STORED(oid, cid, self.nid)
                    .send(&mut self.to_boot)
                    .await?;
            }
            Operation::RETRIEVE => {
                if self.obj.retrieve_object(cid, oid) {
                    Message::OBJ_RETRIEVED(oid, cid, self.nid)
                } else {
                    Message::NOT_FOUND(oid)
                }
                .send(&mut self.to_boot)
                .await?;
            }
        }
        Ok(())
    }

    /// According to chord rules, a node owns an object if
    /// it's between its predecessor and itself.
    fn owns_object(&self, oid: ObjectId) -> bool {
        self.predecessor < oid && oid <= self.nid
    }

    /// Send the request along the ring
    async fn propagate_request(&mut self, req: Message) -> io::Result<()> {
        self.puller.tell_node(req, self.successor).await?;
        Ok(())
    }

    /// Prints the ring information for this node
    pub fn ring_print(&self) {
        eprintln!(
            "Predecessor: n{}, Successor: n{}",
            self.predecessor, self.successor
        );
    }
}
