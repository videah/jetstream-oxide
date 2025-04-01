# jetstream-oxide

[![Crate](https://img.shields.io/crates/v/jetstream-oxide.svg)](https://crates.io/crates/jetstream-oxide)
[![docs.rs](https://docs.rs/jetstream-oxide/badge.svg)](https://docs.rs/jetstream-oxide/latest/jetstream_oxide)

A typed Rust library for easily interacting with and consuming the
Bluesky [Jetstream](https://github.com/bluesky-social/jetstream)
service.

```rust
let config = JetstreamConfig {
    endpoint: DefaultJetstreamEndpoints::USEastOne.into(),
    compression: JetstreamCompression::Zstd,
    ..Default::default()
};

let jetstream = JetstreamConnector::new(config).unwrap();
let receiver = jetstream.connect().await?;

while let Ok(event) = receiver.recv_async().await {
    if let Commit(commit_event) = event {
        match commit_event.commit {
            CommitData::Create { .. } => {
                println!("Received create event: {:#?}", commit_event.info);
            }
            CommitData::Update { .. } => {
                println!("Received update event: {:#?}", commit_event.info);
            }
            CommitData::Delete { .. } => {
                println!("Received delete event: {:#?}", commit_event.info);
            }
        }
    }
}
```

## Example

A small example CLI utility to show how to use this crate can be found in the `examples` directory. To run it, use the
following command:

```sh
cargo run --example basic -- --nsid "app.bsky.feed.post"
```

This will display a real-time feed of every single post that is being made or deleted in the entire Bluesky network,
right in your terminal!

You can filter it down to just specific accounts like this:

```sh
cargo run --example basic -- \
--nsid "app.bsky.feed.post" \
--did "did:plc:inze6wrmsm7pjl7yta3oig77"
```

This listens for posts that _I personally make_. You can substitute your own DID and make a few test posts yourself if
you'd
like of course!
