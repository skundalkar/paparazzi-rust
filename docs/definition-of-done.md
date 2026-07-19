# v1.0 definition of done

`paparazzi-rust` v1.0 is an offline compatibility toolchain, not flight-control
software. It is complete only when every v1 scope item has deterministic tests
and an explicit compatibility state.

| Area | State | Completion evidence |
| --- | --- | --- |
| PPRZ v1 framing | Implemented | Byte-for-byte golden tests and malformed-stream recovery |
| Offline stream replay | Implemented | Unit tests plus 941/941 frames accepted from upstream `pprz.bin` |
| XML message dictionary and primitive decoding | Initial subset | Parser and scalar/array decoder implemented; capture schema provenance remains incomplete |
| Airframe root/firmware/targets, firmware-level modules and defines | Implemented | Bebop-style XML compatibility fixture |
| Sections, target-specific declarations, command laws | Deferred | Not exposed as supported API |
| Hardware I/O, actuator output, firmware flashing | Excluded | No interfaces or dependencies in v1 |

The release gate requires `cargo fmt --check`, strict Clippy, all unit and
exhaustive protocol tests, documented upstream commit provenance, and a
published compatibility matrix like this one.
