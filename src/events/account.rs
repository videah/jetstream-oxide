use chrono::Utc;
use serde::Deserialize;

use crate::{events::EventInfo, exports};

/// An event representing a change to an account.
#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct AccountEvent {
    /// Basic metadata included with every event.
    #[serde(flatten)]
    pub info: EventInfo,
    /// Account specific data bundled with this event.
    pub account: AccountData,
}

/// Account specific data bundled with an account event.
#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct AccountData {
    /// Whether the account is currently active.
    pub active: bool,
    /// The DID of the account.
    pub did: exports::Did,
    pub seq: u64,
    pub time: chrono::DateTime<Utc>,
    /// If `active` is `false` this will be present to explain why the account is inactive.
    pub status: Option<AccountStatus>,
}

/// The possible reasons an account might be listed as inactive.
#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Deactivated,
    Deleted,
    Suspended,
    TakenDown,
}
