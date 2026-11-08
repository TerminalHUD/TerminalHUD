# HudSwitch core tests

`state_machine.rs` is registered as a `[[test]]` target by
`src/hudswitch-core/Cargo.toml`. Run it with `cargo test -p hudswitch-core --locked`
or as part of `cargo test --workspace --locked`.

The suite exercises the public controller contract through a fake backend. It
checks observable visibility/input/focus outcomes, acquisition intent and ordering,
style ownership, stale lifecycle filtering, capability refusals, serialization,
and partial backend failures with safe recovery or retained unconfirmed ownership.
Real OS and terminal tests belong in the separate backend/product suites.
