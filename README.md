# paparazzi-rust

A Rust reimplementation of selected [Paparazzi UAS](https://github.com/paparazzi/paparazzi) components.

## Status

This repository is in its foundation phase. The first goal is compatibility tooling:

- PPRZ v1 message framing and protocol primitives;
- airframe/configuration parsing and validation;
- deterministic math and coordinate primitives; and
- simulator/replay adapters and upstream-vs-Rust differential tests.

It is **not flight-control software** and must not be used to control real aircraft. Hardware-in-the-loop validation, safety analysis, and an explicit release policy are required before any onboard deployment is considered.

## Scope

`paparazzi-rust` is an independent, clean Rust implementation. It will use public Paparazzi documentation, configuration files, protocol definitions, and behavior captured from simulation as compatibility references. Each migrated feature must have a documented upstream baseline and automated equivalence tests.

See [the architecture](docs/architecture.md), [compatibility policy](docs/compatibility.md), and [safety scope](docs/safety-scope.md).
The release boundary and verification gate are in [the v1.0 definition of done](docs/definition-of-done.md).
For the project model, diagrams, requirements, migration phases, and current
feature status, see the [project brief](docs/project-brief.md),
[migration plan](docs/migration-plan.md), and [progress report](docs/progress-report.md).

The first migrated component is an offline PPRZ v1 transport encoder/decoder.
Its reference baseline and wire-format evidence are recorded in
[the upstream baseline](docs/upstream-baseline.md).

## Development

```sh
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

Replay a recorded PPRZ v1 byte stream entirely offline:

```sh
cargo run -p pprz-sim --bin pprz-replay -- path/to/recording.bin
```

Decode a recording against an upstream telemetry dictionary:

```sh
cargo run -p pprz-sim --bin pprz-decode -- path/to/recording.bin path/to/messages.xml
```

## License and attribution

This project is licensed under GPL-2.0-only. Paparazzi UAS is a separate GPL-2.0 project; see [NOTICE](NOTICE) for attribution. This project is not affiliated with or endorsed by the Paparazzi UAS project.
