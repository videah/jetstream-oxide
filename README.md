# jetstream-oxide

A typed Rust library for easily interacting with and consuming the
Bluesky [Jetstream](https://github.com/bluesky-social/jetstream)
service.

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

This listens for posts that *I personally make*. You can substitute your own DID and make a few test posts yourself if
you'd
like of course!