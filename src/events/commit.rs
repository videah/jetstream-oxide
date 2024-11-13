use atrium_api::record::KnownRecord;
use serde::Deserialize;

use crate::{
    events::EventInfo,
    exports,
};

/// An event representing a repo commit, which can be a `create`, `update`, or `delete` operation.
#[derive(Deserialize, Debug)]
#[serde(untagged, rename_all = "snake_case")]
pub enum CommitEvent {
    Create {
        #[serde(flatten)]
        info: EventInfo,
        commit: CommitData,
    },
    Update {
        #[serde(flatten)]
        info: EventInfo,
        commit: CommitData,
    },
    Delete {
        #[serde(flatten)]
        info: EventInfo,
        commit: CommitInfo,
    },
}

/// The type of commit operation that was performed.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CommitType {
    Create,
    Update,
    Delete,
}

/// Basic commit specific info bundled with every event, also the only data included with a `delete`
/// operation.
#[derive(Deserialize, Debug)]
pub struct CommitInfo {
    /// The type of commit operation that was performed.
    pub operation: CommitType,
    pub rev: String,
    pub rkey: String,
    /// The NSID of the record type that this commit is associated with.
    pub collection: exports::Nsid,
}

/// Detailed data bundled with a commit event. This data is only included when the event is
/// `create` or `update`.
#[derive(Deserialize, Debug)]
pub struct CommitData {
    #[serde(flatten)]
    pub info: CommitInfo,
    /// The CID of the record that was operated on.
    pub cid: exports::Cid,
    /// The record that was operated on.
    pub record: KnownRecord,
}
