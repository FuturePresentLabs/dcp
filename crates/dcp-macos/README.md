# dcp-macos

Loopback-only DCP provider for bounded local macOS controls. It discovers
installed `.app` bundles at catalog time and currently publishes:

- `macos.app.open`
- `macos.audio.volume.set`
- `macos.audio.mute.set`

Run with `cargo run -p dcp-macos`. The default endpoint is
`http://127.0.0.1:18841`; override it with `DCP_MACOS_BIND`.

The provider does not route through Canvas and never accepts shell text. App
IDs come from the live catalog, numeric volume is range-checked, and execution
revalidates catalog and state revisions before invoking fixed macOS binaries.
