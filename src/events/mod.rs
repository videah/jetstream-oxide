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
