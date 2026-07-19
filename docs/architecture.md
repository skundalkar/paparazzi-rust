# Architecture

The workspace is divided by stable compatibility boundaries rather than by the
upstream directory layout.

| Crate | Responsibility | Initial safety boundary |
| --- | --- | --- |
| `pprz-protocol` | PPRZ v1 frame encoding and stream decoding | No transport or hardware I/O |
| `pprz-messages` | Message XML dictionaries and typed payload decoding | Offline interpretation only |
| `pprz-config` | Airframe and system configuration parsing | Parse/validate only |
| `pprz-math` | Deterministic math and coordinate primitives | Pure functions only |
| `pprz-sim` | Offline replay and simulation adapters | No actuator or serial interfaces |

Onboard deployment is not part of the initial architecture. It may be proposed
only after a component has compatibility coverage, simulation evidence, an
independent safety review, and an explicit release decision.

The complete component model, data structures, and diagrams are in the
[project brief](project-brief.md); current implementation status is in the
[progress report](progress-report.md).
