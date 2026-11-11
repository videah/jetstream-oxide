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
///
/// The lexicon (`com.atproto.sync.subscribeRepos#account`) defines `status` as a string with
/// `knownValues`, which is an open set: new values can appear at any time. Values this library
/// does not know about are preserved in [`AccountStatus::Unknown`] rather than failing to parse
/// (which would otherwise drop the whole event and the connection with it).
#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Deactivated,
    Deleted,
    Suspended,
    TakenDown,
    Desynchronized,
    Throttled,
    /// A status not (yet) known to this library, holding the raw value.
    #[serde(untagged)]
    Unknown(String),
}

#[cfg(test)]
mod test {
    use super::AccountStatus;

    fn parse(status: &str) -> AccountStatus {
        serde_json::from_str(&format!("\"{status}\"")).unwrap()
    }

    #[test]
    fn parse_known_statuses() {
        assert_eq!(parse("deactivated"), AccountStatus::Deactivated);
        assert_eq!(parse("deleted"), AccountStatus::Deleted);
        assert_eq!(parse("suspended"), AccountStatus::Suspended);
        assert_eq!(parse("takendown"), AccountStatus::TakenDown);
        assert_eq!(parse("desynchronized"), AccountStatus::Desynchronized);
        assert_eq!(parse("throttled"), AccountStatus::Throttled);
    }

    #[test]
    fn parse_unknown_status() {
        assert_eq!(
            parse("some-future-status"),
            AccountStatus::Unknown("some-future-status".to_string())
        );
    }
}
