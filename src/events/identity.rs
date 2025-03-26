use chrono::Utc;
use serde::Deserialize;

use crate::{events::EventInfo, exports};

/// An event representing a change to an identity.
#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct IdentityEvent {
    /// Basic metadata included with every event.
    #[serde(flatten)]
    pub info: EventInfo,
    /// Identity specific data bundled with this event.
    pub identity: IdentityData,
}

/// Identity specific data bundled with an identity event.
#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct IdentityData {
    /// The DID of the identity.
    pub did: exports::Did,
    /// The handle associated with the identity.
    pub handle: Option<exports::Handle>,
    pub seq: u64,
    pub time: chrono::DateTime<Utc>,
}
