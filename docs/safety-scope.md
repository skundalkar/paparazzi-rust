# Safety scope

`paparazzi-rust` is experimental software. It must not be used to operate real
aircraft, actuators, or safety-critical ground equipment.

Allowed work in the foundation phase:

- offline parsing and serialization;
- deterministic unit and property tests;
- recorded telemetry replay; and
- software-in-the-loop simulation.

Excluded work:

- firmware flashing;
- direct serial/CAN actuator control;
- autonomous flight; and
- any claim of flight readiness or certification.
