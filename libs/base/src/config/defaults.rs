//! The servers and the password this build uses.
//!
//! The servers are installed as *defaults* of `Config::get_option`, so a user can still point the
//! client elsewhere from Settings and nothing here is ever written into the config file.
//!
//! The password is installed as the hard/preset one (its hash, not the plaintext), so a session is
//! accepted with it on every device of this deployment without anyone typing anything, and the
//! random temporary password is out of the picture.

use crate::config::keys;
use hbb_common::{
    config::{
        compute_permanent_password_h1, BUILTIN_SETTINGS, DEFAULT_SETTINGS, HARD_SETTINGS,
        OVERWRITE_SETTINGS,
    },
    sodiumoxide::base64,
};

/// ID (rendezvous) server.
pub const ID_SERVER: &str = "rustdesk.aichen.fun:21116";

/// Relay server.
pub const RELAY_SERVER: &str = "rustdesk.aichen.fun:21117";

/// API server.
pub const API_SERVER: &str = "http://rustdesk.aichen.fun:21114";

/// Public key of the server, used to verify the ID and relay signatures.
pub const KEY: &str = "Y0tAtNMbwTsNRxEVnuOrZWsn6Mr8XjCrBJnnWKVi4WI=";

/// The password of every session, for both directions.
pub const PASSWORD: &str = "fdh92672QQ@";

/// Salt the preset password is hashed with. A peer receives it in the login hash, so a constant
/// costs nothing: what never leaves this build is the hash of the two.
const PASSWORD_SALT: &str = "rustdesk.aichen.fun";

/// The prefix `hbb_common` puts in front of a hashed (not encrypted) permanent password.
const PASSWORD_HASH_PREFIX: &str = "00";

/// Install the servers, the password and the session defaults. Call once, before anything reads
/// an option or authenticates a peer in this process.
pub fn apply() {
    apply_servers();
    apply_session();
    apply_password();
}

fn apply_servers() {
    let mut defaults = DEFAULT_SETTINGS.write().unwrap();
    defaults.insert(
        keys::OPTION_CUSTOM_RENDEZVOUS_SERVER.to_owned(),
        ID_SERVER.to_owned(),
    );
    defaults.insert(keys::OPTION_RELAY_SERVER.to_owned(), RELAY_SERVER.to_owned());
    defaults.insert(keys::OPTION_API_SERVER.to_owned(), API_SERVER.to_owned());
    defaults.insert(keys::OPTION_KEY.to_owned(), KEY.to_owned());
}

/// A machine that ran RustDesk before this build keeps its stored options, and a stored option
/// outranks a default, so these three are installed as overwrites: whatever an older install left
/// behind, the session still takes the fixed password, still needs no click, and still gets every
/// permission. They show up as fixed in Settings, which is the point.
fn apply_session() {
    let mut overwrite = OVERWRITE_SETTINGS.write().unwrap();
    // Every permission, so a session starts with full access.
    overwrite.insert(keys::OPTION_ACCESS_MODE.to_owned(), "full".to_owned());
    // The fixed password is the only way in: no temporary (random) password is accepted.
    overwrite.insert(
        keys::OPTION_VERIFICATION_METHOD.to_owned(),
        "use-permanent-password".to_owned(),
    );
    overwrite.insert(keys::OPTION_APPROVE_MODE.to_owned(), "password".to_owned());
    drop(overwrite);

    let mut builtin = BUILTIN_SETTINGS.write().unwrap();
    // Connections made *from* this build carry the password with them, so they get in without
    // anyone typing it either.
    builtin.insert(
        keys::OPTION_DEFAULT_CONNECT_PASSWORD.to_owned(),
        PASSWORD.to_owned(),
    );
    builtin.insert(
        keys::OPTION_DISABLE_CHANGE_PERMANENT_PASSWORD.to_owned(),
        "Y".to_owned(),
    );
}

fn apply_password() {
    let h1 = compute_permanent_password_h1(PASSWORD, PASSWORD_SALT);
    let storage = PASSWORD_HASH_PREFIX.to_owned() + &base64::encode(&h1, base64::Variant::Original);
    let mut hard = HARD_SETTINGS.write().unwrap();
    hard.insert("password".to_owned(), storage);
    hard.insert("salt".to_owned(), PASSWORD_SALT.to_owned());
}
