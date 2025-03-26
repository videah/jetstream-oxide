use atrium_api::record::KnownRecord;
use serde::Deserialize;

use crate::{events::EventInfo, exports};

/// An event representing a repo commit.
#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "snake_case")]
pub struct CommitEvent {
    #[serde(flatten)]
    pub info: EventInfo,
    pub commit: CommitData,
}

/// Basic commit specific info bundled with every event, also the only data included with a `delete`
/// operation.
#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct CommitInfo {
    pub rev: String,
    pub rkey: String,
    /// The NSID of the record type that this commit is associated with.
    pub collection: exports::Nsid,
}

/// Detailed data bundled with a commit event.
#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum CommitData {
    Create {
        #[serde(flatten)]
        info: CommitInfo,
        /// The CID of the record that was operated on.
        cid: exports::Cid,
        /// The record that was operated on.
        record: KnownRecord,
    },
    Update {
        #[serde(flatten)]
        info: CommitInfo,
        /// The CID of the record that was operated on.
        cid: exports::Cid,
        /// The record that was operated on.
        record: KnownRecord,
    },
    Delete {
        #[serde(flatten)]
        info: CommitInfo,
    },
}
