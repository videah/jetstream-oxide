//! A very basic example of how to listen for create/delete events on a specific DID and NSID.

use atrium_api::{record::KnownRecord::AppBskyFeedPost, types::string};
use clap::Parser;
use jetstream_oxide::{
    events::{commit::CommitData, JetstreamEvent::Commit},
    DefaultJetstreamEndpoints, JetstreamCompression, JetstreamConfig, JetstreamConnector,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The DIDs to listen for events on, if not provided we will listen for all DIDs.
    #[arg(short, long)]
    did: Option<Vec<string::Did>>,
    /// The NSID for the collection to listen for (e.g. `app.bsky.feed.post`).
    #[arg(short, long)]
    nsid: string::Nsid,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let dids = args.did.unwrap_or_default();
    let config = JetstreamConfig {
        endpoint: DefaultJetstreamEndpoints::USEastOne.into(),
        wanted_collections: vec![args.nsid.clone()],
        wanted_dids: dids.clone(),
        compression: JetstreamCompression::Zstd,
        cursor: None,
        max_retries: 10,
        max_delay_ms: 30_000,
        base_delay_ms: 1_000,
        reset_retries_min_ms: 30_000,
    };

    let jetstream = JetstreamConnector::new(config)?;
    let receiver = jetstream.connect().await?;

    println!(
        "Listening for '{}' events on DIDs: {:?}",
        args.nsid.to_string(),
        dids,
    );

    while let Ok(event) = receiver.recv_async().await {
        if let Commit(event) = event {
            match event.commit {
                CommitData::Create {
                    info,
                    cid: _cid,
                    record: AppBskyFeedPost(record),
                } => {
                    println!("New post created! ({})\n\n'{}'", info.rkey, record.text);
                }
                CommitData::Delete { info } => {
                    println!("A post has been deleted. ({})", info.rkey);
                }
                _ => {}
            }
        }
    }

    Ok(())
}
