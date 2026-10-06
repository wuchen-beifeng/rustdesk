# What this fork changes

`master` tracks `rustdesk/rustdesk@master` and adds exactly nine files of its own.
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
| `.github/workflows/upstream-sync-check.yml` | Daily report of how far upstream has moved and whether the next merge conflicts. Never merges. |

## The deployment

`defaults.rs` holds the constants that describe a deployment:

```rust
pub const ID_SERVER: &str = "rustdesk.aichen.fun:21116";
pub const RELAY_SERVER: &str = "rustdesk.aichen.fun:21117";
pub const API_SERVER: &str = "http://rustdesk.aichen.fun:21114";
pub const KEY: &str = "<server public key>";
pub const PASSWORD: &str = match option_env!("RUSTDESK_PASSWORD") { /* ... */ };
```

The servers and the server key are in the file on purpose: they are what a client
must know to reach this deployment, so they are public by construction and a build
without them is useless. The *password* is not: it is compiled in from the
`RUSTDESK_PASSWORD` environment variable, which the workflow takes from the
repository secret of the same name, so the plaintext is in neither the repository
nor a config file. `RUSTDESK_PASSWORD=... python3 build.py --portable ...` does the
same for a local build.

An empty password is not a password: `apply()` installs no preset and, because
`verification-method` is `use-permanent-password`, such a build accepts no session
at all rather than falling back to a random temporary password. The workflow's guard
step refuses to build in that state.

The servers are installed as *defaults* of `Config::get_option`, so they are never
written into a config file and a user can still point a client elsewhere from
Settings. The password is installed as the hard/preset one: `HARD_SETTINGS["salt"]`
plus `HARD_SETTINGS["password"] = "00" + base64(sha256(plaintext + salt))`. The
plaintext never reaches the config file, and the server hands the salt to the peer
inside the login hash, so a constant salt costs nothing.

Note that the password is still in this repository's *history* (it was committed
before it became a secret) and inside every exe built from it. Removing it from the
source stops new leaks; it does not unpublish the old ones. Rotating it means
rebuilding and redistributing the clients.

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

## Why `validate_password()` is patched at all

The preset password is not enough on its own. A machine that ran RustDesk before this
build keeps its own permanent password in its config, and when one is stored
`Config::get_effective_permanent_password_salt()` returns the *local* salt and
`validate_password()` never looks at the preset: the fixed password would be locked
out. Hence the six lines at the top of `validate_password()` that accept the fixed
password first.

The alternative -- clearing the stored password once at startup from `defaults.rs`,
which would leave `src/server/connection.rs` untouched -- was rejected: a config sync
from the server can write a stored password again *while the process runs*, and then
the fixed password stops working until the next restart. Accepting it first cannot be
out-raced. `validate_password_plain("")` returns `false`, so the patch is also inert
when no password was compiled in.

## Things that are deliberately left alone

* `unlock_pin` — a local UI unlock PIN, not connection authentication.
* The random salt/challenge from `Config::get_auto_password` — part of the protocol
  handshake, not a password.
* TOTP — only exists when a Pro server sends `require_2fa`.
* `libs/hbb_common/**` — a git submodule. The CI checks out the commit recorded in
  the parent repository, so an edit inside the submodule is not built. Use its public
  API instead (`hbb_common::config::*`, `hbb_common::sodiumoxide::*`).

## Knowing when upstream moved

`.github/workflows/upstream-sync-check.yml` runs daily (and on demand) and reports, in
the run summary, how many commits behind upstream master this fork is, which files
both sides changed since the merge base, and the result of a trial merge -- the only
honest answer to "will the next sync conflict". It reads only: nothing is merged,
committed or pushed, and the job succeeds either way.

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
