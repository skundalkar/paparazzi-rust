# Upstream baseline

The first compatibility baseline is the Paparazzi `master` commit:

```text
f43dc86f5130e0deb03d0f0206e72b37ca8a97c5
```

The initial migration target is the PPRZ v1 transport framing behavior in:

```text
sw/tools/tcp_aircraft_server/phoenix/pprz_transport.py
```

That reference defines a frame as:

```text
| STX (0x99) | length | aircraft ID | message ID | payload | checksum A | checksum B |
```

`length` is the full frame length, including all six non-payload bytes. The
two checksum accumulators both begin at zero; they consume `length`, aircraft
ID, message ID, and payload in order using wrapping 8-bit addition. The
reference frame used by the Rust golden tests is:

```text
99 08 2A 07 01 02 3C E9
```

This milestone is limited to offline encoding, decoding, and recovery from a
malformed frame. It has no transport, hardware, or actuator interface.

The decoder was additionally checked against the public upstream recording
`sw/misc/log_parser/pprz.bin` at that commit: all 941 encoded frames were
accepted with zero checksum rejections. Its first frame is retained as a Rust
golden test without copying the recording into this repository.

## Airframe configuration subset

The second migration target is the airframe XML shape represented by
`conf/airframes/examples/bebop.xml` at the same upstream commit. The initial
Rust parser reads the airframe name, the first firmware declaration, its
declared targets (for example, `ap`/`bebop` and `nps`/`pc`), and firmware- and
target-level module and define declarations. It does not yet evaluate sections
or command laws.

## Message dictionary subset

The third migration target is Paparazzi's XML message-dictionary shape. The
Rust `pprz-messages` crate parses one named class, validates scalar and
variable-array layouts, and decodes the supported primitive values from an
offline PPRZ frame. Its initial schema reference is
`sw/logalizer/matlab_log/messages.xml` at the pinned commit.

That dictionary is historical and does **not** fully describe the separately
recorded `sw/misc/log_parser/pprz.bin`: 42 of 941 transport-valid frames decode
under it. The remaining length mismatches are retained as a compatibility gap,
not treated as malformed transport data. A matching dictionary or generated
message artifacts must be pinned before those payloads can be claimed typed
compatible.
