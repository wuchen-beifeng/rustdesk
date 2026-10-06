# What this fork changes

`master` tracks `rustdesk/rustdesk@master` and adds exactly seven files of its own.
Everything else is upstream. Synced with upstream at `9f9585ce` (see the merge
commit on `master`); `git diff upstream/master..master --stat` is the whole delta.

| File | Change |
|---|---|
| `libs/base/src/config/defaults.rs` | New. The ID/relay/API servers, the server public key, the fixed password and the session defaults; `apply()` installs them. |
| `libs/base/src/config/mod.rs` | Exports the module above. |
| `src/common.rs` | `global_init()` calls `defaults::apply()`; `get_key()` falls back to `defaults::KEY` instead of the official RustDesk key. |
| `src/server/connection.rs` | `validate_password()` accepts the fixed password before it looks at any stored one. |
| `flutter/lib/main.dart` | `runConnectionManagerScreen()` hides the connection manager window instead of consulting `hide_cm`. |
| `flutter/lib/models/server_model.dart` | `hideCm` starts as `true`, so `_addTab()` never raises the window. |
| `.github/workflows/build-windows-exe.yml` | The Windows portable-exe build (manual dispatch). |

## The deployment

`defaults.rs` holds the four constants that describe a deployment:

```rust
pub const ID_SERVER: &str = "rustdesk.aichen.fun:21116";
pub const RELAY_SERVER: &str = "rustdesk.aichen.fun:21117";
pub const API_SERVER: &str = "http://rustdesk.aichen.fun:21114";
pub const KEY: &str = "<server public key>";
pub const PASSWORD: &str = "<the fixed password>";
```

The servers are installed as *defaults* of `Config::get_option`, so they are never
written into a config file and a user can still point a client elsewhere from
Settings. The password is installed as the hard/preset one: `HARD_SETTINGS["salt"]`
plus `HARD_SETTINGS["password"] = "00" + base64(sha256(plaintext + salt))`. The
plaintext never reaches the config file, and the server hands the salt to the peer
inside the login hash, so a constant salt costs nothing.

Three options are installed as *overwrites* rather than defaults, because a stored
option outranks a default and a machine that ran RustDesk before this build keeps
its stored options:

* `access-mode = "full"` — every permission is enabled locally, so a session starts
  with keyboard, clipboard, files, audio, camera, terminal, tunnel and the rest.
* `verification-method = "use-permanent-password"` — the random temporary password
  is neither accepted nor shown.
* `approve-mode = "password"` — an incoming session needs no click.

They show up as fixed in Settings, which is the point. Two builtin options come
with it: `default-connect-password` (the same password, so *outbound* connections
do not ask either) and `disable-change-permanent-password = "Y"`.

## The hidden connection-manager window

Upstream raises the connection-manager window on every incoming session
(`windowOnTop()` in `ServerModel._addTab`), and its `hide_cm` switch only returns
`true` for a pro/custom client, never for an open-source build. This fork hides the
window at startup unconditionally.

Consequence: a request that needs a click cannot be confirmed and times out. That is
why the fixed password and `approve-mode = "password"` are not optional here — they
are what keeps an unattended session working with no window to click in.

## Things that are deliberately left alone

* `unlock_pin` — a local UI unlock PIN, not connection authentication.
* The random salt/challenge from `Config::get_auto_password` — part of the protocol
  handshake, not a password.
* TOTP — only exists when a Pro server sends `require_2fa`.
* `libs/hbb_common/**` — a git submodule. The CI checks out the commit recorded in
  the parent repository, so an edit inside the submodule is not built. Use its public
  API instead (`hbb_common::config::*`, `hbb_common::sodiumoxide::*`).

## Syncing with upstream

```bash
git fetch upstream master
git merge upstream/master          # expected conflicts: src/common.rs, src/server/connection.rs
```

Both of those files are patched in a few lines that upstream also edits, so a
conflict there is normal: keep upstream's new code and re-apply this fork's hunk on
top. `defaults.rs`, `main.dart` and `server_model.dart` are additive and rarely
conflict. Then run the workflow below and check that its verification step passes.

## Building the exe

`.github/workflows/build-windows-exe.yml` (manual dispatch) runs three jobs:
`generate-bridge` (Flutter 3.22.3 + flutter_rust_bridge 1.80.1 — it needs
`components: "rustfmt"`, the generator formats what it writes), `build-topmost`
(WindowInjection.dll) and `build-windows-exe` (windows-2022, LLVM 15, Flutter
3.24.5 with the rustdesk engine, vcpkg `x64-windows-static`, `build.py --portable
--flutter --skip-portable-pack --hwcodec --vram`, then `libs/portable/generate.py`).

The artifact is named after the version in `Cargo.toml`. Before packing, the job
reads `ID_SERVER` and `KEY` out of `defaults.rs` and requires them — and the literal
`use-permanent-password` — to be present in `librustdesk.dll`, so a sync that quietly
drops `defaults::apply()` fails the build instead of shipping a client that still
talks to the official servers.

Note that the password and the server key end up inside the exe: the exe is a
credential. The workflow only proves that the customisations are compiled in, not
that the client works against a live server.
