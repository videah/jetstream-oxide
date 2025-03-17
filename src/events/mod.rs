pub mod account;
pub mod commit;
pub mod identity;

use serde::Deserialize;

use crate::exports;

/// Basic data that is included with every event.
#[derive(Deserialize, Debug)]
pub struct EventInfo {
    pub did: exports::Did,
    pub time_us: u64,
    pub kind: EventKind,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum JetstreamEvent {
    Commit(commit::CommitEvent),
    Identity(identity::IdentityEvent),
    Account(account::AccountEvent),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Commit,
    Identity,
    Account,
}

#[cfg(test)]
mod test {
    use super::*;

    const UPDATE_EVENT: &str = r#"{
          "did": "did:plc:z72i7hdynmk6r22z27h6tvur",
          "time_us": 1742234149201771,
          "kind": "commit",
          "commit": {
            "rev": "3lklpzt6jvf2n",
            "operation": "update",
            "collection": "app.bsky.actor.profile",
            "rkey": "self",
            "record": { "$type": "app.bsky.actor.profile" },
            "cid": "bafyreie4x2nfhr7zbeuwfuad7ess3mlpudgab7cqw3b4rqiormnq726spu"
          }
        }
    "#;

    #[test]
    fn regression_update_parsed_as_create() {
        let parsed: JetstreamEvent = serde_json::from_str(UPDATE_EVENT).unwrap();
        let JetstreamEvent::Commit(c) = parsed else {
            panic!("expected a commit event, found {:?}", parsed);
        };
        let commit::CommitEvent::Update { .. } = c else {
            panic!("expected an update commit, found {:?}", c);
        };
    }
}
