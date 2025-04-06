use std::{thread::sleep, time::Duration};

use clap::Parser;
use setup::hostsfile::Objects;
use state::{bootstrap::Ring, messaging::Message, Peer};

mod args;
mod setup;
mod state;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let arguments = args::Project5::parse();

    if arguments.is_bootstrap() {
        // Bootstrap Code
        let mut ring = Ring::new().await?;
        println!("Ring is set up");
        loop {
            if let Some(msg) = ring.poll() {
                match msg {
                    Message::Join(nid) => ring.respond_to_join(nid).await?,
                    _ => {}
                }
            }
        }
    } else {
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
}
