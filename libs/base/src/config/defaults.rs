//! The servers this build talks to.
//!
//! They are installed as *defaults* of `Config::get_option`, so a user can still point the
//! client elsewhere from Settings and nothing here is ever written into the config file.

use crate::config::keys;
use hbb_common::config::DEFAULT_SETTINGS;

/// ID (rendezvous) server.
pub const ID_SERVER: &str = "rustdesk.aichen.fun:21116";

/// Relay server.
pub const RELAY_SERVER: &str = "rustdesk.aichen.fun:21117";

/// API server.
pub const API_SERVER: &str = "http://rustdesk.aichen.fun:21114";

/// Public key of the server, used to verify the ID and relay signatures.
pub const KEY: &str = "Y0tAtNMbwTsNRxEVnuOrZWsn6Mr8XjCrBJnnWKVi4WI=";

/// Install the servers above as defaults. Call once, before any option is read in this process.
pub fn apply() {
    let mut defaults = DEFAULT_SETTINGS.write().unwrap();
    defaults.insert(
        keys::OPTION_CUSTOM_RENDEZVOUS_SERVER.to_owned(),
        ID_SERVER.to_owned(),
    );
    defaults.insert(keys::OPTION_RELAY_SERVER.to_owned(), RELAY_SERVER.to_owned());
    defaults.insert(keys::OPTION_API_SERVER.to_owned(), API_SERVER.to_owned());
    defaults.insert(keys::OPTION_KEY.to_owned(), KEY.to_owned());
}
