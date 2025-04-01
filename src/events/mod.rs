pub mod account;
pub mod commit;
pub mod identity;

use serde::Deserialize;

use crate::exports;

/// Basic data that is included with every event.
#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct EventInfo {
    pub did: exports::Did,
    pub time_us: u64,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JetstreamEvent {
    Commit(commit::CommitEvent),
    Identity(identity::IdentityEvent),
    Account(account::AccountEvent),
}

#[cfg(test)]
mod test {

    use super::{commit::CommitData, *};
    use std::str::FromStr;

    #[test]
    fn parse_commit_update() {
        let event_json: &str = r#"{
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

        let event_expected_object = JetstreamEvent::Commit(commit::CommitEvent {
            info: EventInfo {
                did: exports::Did::new("did:plc:z72i7hdynmk6r22z27h6tvur".to_string()).unwrap(),
                time_us: 1742234149201771,
            },
            commit: CommitData::Update {
                info: commit::CommitInfo {
                    rev: "3lklpzt6jvf2n".into(),
                    rkey: "self".into(),
                    collection: exports::Nsid::new("app.bsky.actor.profile".to_string()).unwrap(),
                },
                cid: exports::Cid::from_str(
                    "bafyreie4x2nfhr7zbeuwfuad7ess3mlpudgab7cqw3b4rqiormnq726spu",
                )
                .unwrap(),
                record: serde_json::from_str(r#"{ "$type": "app.bsky.actor.profile" }"#).unwrap(),
            },
        });

        let parsed: JetstreamEvent = serde_json::from_str(event_json).unwrap();

        assert_eq!(parsed, event_expected_object);
    }

    #[test]
    fn parse_commit_create() {
        let event_json: &str = r#"{
            "did": "did:plc:z72i7hdynmk6r22z27h6tvur",
            "time_us": 1742234149201771,
            "kind": "commit",
            "commit": {
              "rev": "3lklpzt6jvf2n",
              "operation": "create",
              "collection": "app.bsky.actor.profile",
              "rkey": "self",
              "record": { "$type": "app.bsky.actor.profile" },
              "cid": "bafyreie4x2nfhr7zbeuwfuad7ess3mlpudgab7cqw3b4rqiormnq726spu"
            }
          }
        "#;

        let event_expected_object = JetstreamEvent::Commit(commit::CommitEvent {
            info: EventInfo {
                did: exports::Did::new("did:plc:z72i7hdynmk6r22z27h6tvur".to_string()).unwrap(),
                time_us: 1742234149201771,
            },
            commit: CommitData::Create {
                info: commit::CommitInfo {
                    rev: "3lklpzt6jvf2n".into(),
                    rkey: "self".into(),
                    collection: exports::Nsid::new("app.bsky.actor.profile".to_string()).unwrap(),
                },
                cid: exports::Cid::from_str(
                    "bafyreie4x2nfhr7zbeuwfuad7ess3mlpudgab7cqw3b4rqiormnq726spu",
                )
                .unwrap(),
                record: serde_json::from_str(r#"{ "$type": "app.bsky.actor.profile" }"#).unwrap(),
            },
        });

        let parsed: JetstreamEvent = serde_json::from_str(event_json).unwrap();
        assert_eq!(parsed, event_expected_object);
    }

    #[test]
    fn parse_commit_delete() {
        let event_json: &str = r#"{
            "did": "did:plc:rfov6bpyztcnedeyyzgfq42k",
            "time_us": 1725516666833633,
            "kind": "commit",
            "commit": {
              "rev": "3l3f6nzl3cv2s",
              "operation": "delete",
              "collection": "app.bsky.graph.follow",
              "rkey": "3l3dn7tku762u"
            }
          }
        "#;

        let event_expected_object = JetstreamEvent::Commit(commit::CommitEvent {
            info: EventInfo {
                did: exports::Did::new("did:plc:rfov6bpyztcnedeyyzgfq42k".to_string()).unwrap(),
                time_us: 1725516666833633,
            },
            commit: CommitData::Delete {
                info: commit::CommitInfo {
                    rev: "3l3f6nzl3cv2s".into(),
                    rkey: "3l3dn7tku762u".into(),
                    collection: exports::Nsid::new("app.bsky.graph.follow".to_string()).unwrap(),
                },
            },
        });

        let parsed: JetstreamEvent = serde_json::from_str(event_json).unwrap();
        assert_eq!(parsed, event_expected_object);
    }

    #[test]
    fn parse_account() {
        let event_json: &str = r#"{
            "did": "did:plc:z72i7hdynmk6r22z27h6tvur",
            "time_us": 1742234149201771,
            "kind": "account",
            "account": {
              "active": true,
              "did": "did:plc:z72i7hdynmk6r22z27h6tvur",
              "seq": 1,
              "time": "2021-09-01T00:00:00Z"
            }
          }
        "#;

        let event_expected_object = JetstreamEvent::Account(account::AccountEvent {
            info: EventInfo {
                did: exports::Did::new("did:plc:z72i7hdynmk6r22z27h6tvur".to_string()).unwrap(),
                time_us: 1742234149201771,
            },
            account: account::AccountData {
                active: true,
                did: exports::Did::new("did:plc:z72i7hdynmk6r22z27h6tvur".to_string()).unwrap(),
                seq: 1,
                time: chrono::DateTime::parse_from_rfc3339("2021-09-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                status: None,
            },
        });

        let parsed: JetstreamEvent = serde_json::from_str(event_json).unwrap();
        assert_eq!(parsed, event_expected_object);
    }

    #[test]
    fn parse_identity() {
        let event_json: &str = r#"{
            "did": "did:plc:4ysnxi6vujpjhovgtn5k4ztr",
            "time_us": 1742234149201771,
            "kind": "identity",
            "identity": {
              "did": "did:plc:4ysnxi6vujpjhovgtn5k4ztr",
              "handle": "sergio.bsky.col.social",
              "seq": 1,
              "time": "2021-09-01T00:00:00Z"
            }
          }
        "#;

        let event_expected_object = JetstreamEvent::Identity(identity::IdentityEvent {
            info: EventInfo {
                did: exports::Did::new("did:plc:4ysnxi6vujpjhovgtn5k4ztr".to_string()).unwrap(),
                time_us: 1742234149201771,
            },
            identity: identity::IdentityData {
                did: exports::Did::new("did:plc:4ysnxi6vujpjhovgtn5k4ztr".to_string()).unwrap(),
                handle: Some(exports::Handle::new("sergio.bsky.col.social".to_string()).unwrap()),
                seq: 1,
                time: chrono::DateTime::parse_from_rfc3339("2021-09-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            },
        });

        let parsed: JetstreamEvent = serde_json::from_str(event_json).unwrap();
        assert_eq!(parsed, event_expected_object);
    }
}
