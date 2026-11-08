# HudSwitch core tests

Portable black-box state-machine and fake-backend tests live here. Register each
real suite with a `[[test]]` target in `src/hudswitch-core/Cargo.toml` so
`cargo test --workspace` runs it. No state-machine tests exist in the foundation.
