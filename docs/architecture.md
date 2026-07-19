# Architecture

The workspace is divided by stable compatibility boundaries rather than by the
upstream directory layout.

| Crate | Responsibility | Initial safety boundary |
| --- | --- | --- |
| `pprz-protocol` | Frame/message codecs and protocol dictionaries | No transport or hardware I/O |
| `pprz-config` | Airframe and system configuration parsing | Parse/validate only |
| `pprz-math` | Deterministic math and coordinate primitives | Pure functions only |
| `pprz-sim` | Offline replay and simulation adapters | No actuator or serial interfaces |

Onboard deployment is not part of the initial architecture. It may be proposed
only after a component has compatibility coverage, simulation evidence, an
independent safety review, and an explicit release decision.
