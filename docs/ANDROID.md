# Android / Omi integration

The repository is `lib-t3-rust`; the Cargo package and Rust imports remain
`t3-protocol` / `t3_protocol`, and the native archive remains `libt3_protocol.a`.
The API was extracted from Omarchy T3 Code without renaming symbols.

## Build

Install Rust, the Android NDK and `cargo-ndk` on your build host. Use the same NDK
and Android API level as Omi. For example, after configuring `ANDROID_NDK_HOME`:

```sh
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk --locked
cargo ndk -t arm64-v8a -t x86_64 build --release --locked
```

The archives are `target/aarch64-linux-android/release/libt3_protocol.a` and
`target/x86_64-linux-android/release/libt3_protocol.a`. Link each into the matching
ABI of Omi's JNI shared library; add `include/` to its C/C++ include path. Follow
the Rust linker output for required Android system libraries. Desktop Linux
ARM64/x86_64 archives cannot be linked into Android, even on the same CPU.

The checked-in C header is the ownership contract. A JNI shim needs only these
functions for synchronous HTTP/RPC operations:

- `t3_client_new()` returns an opaque handle.
- `t3_client_call(handle, utf8_bytes, length)` returns owned UTF-8 JSON.
- `t3_string_free(response)` frees that response exactly once.
- `t3_client_free(handle)` destroys the handle after all work completes.

Use Kotlin byte arrays / JNI byte arrays for UTF-8 JSON. JNI's modified-UTF-8
string helpers must not be mistaken for arbitrary UTF-8 transport. Copy the
result into a managed byte array before freeing the Rust response. Serialize
handle ownership; never close it while another call is active. Calls can block
and must run outside Android's main/UI thread. Do not automatically repeat a
mutation after `rpc_outcome_unknown`; reconcile using stable native command IDs.

## Platform responsibilities

Omi owns login, Android Keystore integration, endpoint discovery, permissions,
lifecycle, background execution and the UI. Supply an existing current T3 cloud
JWT to the relay flow; a browser token bound to a different DPoP key cannot be
reused. Keep tokens and private keys out of logs and UI state. For direct LAN
HTTP, Android's network-security policy must explicitly permit the intended
endpoint; do not disable certificate validation for HTTPS.

The Rust API also exposes subscriptions, attachment upload/download and portable
presentation helpers. The current JSON C ABI exposes only `connectBearer`,
`connectRelay`, `request`, `rpc` and `disconnect`; it does **not** yet expose live
subscription callbacks, streaming attachments or every presentation helper.
Those require a Rust JNI wrapper or an extension of the shared ABI, rather than
reimplementing the protocol in Kotlin. Relay DPoP keys currently live for the C
handle's lifetime; the Rust API supports secure host-managed key persistence.

## Validation status

Native Linux tests cover the protocol, relay/auth flow, streaming and C ownership
contract. The C smoke test links the actual static archive. Android NDK linking,
JNI integration and device/lifecycle validation remain required in Omi. This
extraction does not claim to build or modify the Android application.
