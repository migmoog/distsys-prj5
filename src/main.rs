use std::{thread::sleep, time::Duration};

use clap::Parser;
use setup::hostsfile::Objects;
use state::{
    bootstrap::{Client, Ring},
    messaging::Message,
    Peer,
};

mod args;
mod setup;
mod state;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let arguments = args::Project5::parse();

    // Bootstrap Code
    if arguments.is_bootstrap() {
        let mut ring = Ring::new().await?;
        loop {
            if let Some(msg) = ring.poll() {
                match msg {
                    Message::Join(nid) => ring.respond_to_join(nid).await?,
                    Message::REQUEST(_, _, _, _) => ring.drop_request(msg).await?,
                    _ if msg.is_obj_response() => ring.respond_to_client(msg).await?,
                    _ => {}
                }
            }
        }
    }

    // Client code
    if let Some(tno) = arguments.testcase {
        let mut client = Client::new(arguments.bootstrap.unwrap()).await?;
        if let Some(dur) = arguments.delay {
            sleep(Duration::from_secs(dur));
            client.send_req(tno).await?;
        }

        loop {
            // May or may not need this idc
            client.wait_for_res().await?;
        }
    }

    // Peer code
    let bootstrap_name = arguments
        .bootstrap
        .expect("Peers should know what bootstrap is");
    let objects = Objects::load(arguments.object.unwrap())?;
    let mut peer = Peer::new(objects, bootstrap_name).await?;
    if let Some(dur) = arguments.delay {
        sleep(Duration::from_secs(dur));
        peer.join().await?;
    }
    loop {
        // TODO
        peer.hear().await?;
    }
}
