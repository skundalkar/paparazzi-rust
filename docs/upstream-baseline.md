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
two checksum accumulators both begin at `STX`; they consume `length`, aircraft
ID, message ID, and payload in order using wrapping 8-bit addition. The
reference frame used by the Rust golden tests is:

```text
99 08 2A 07 01 02 D5 7F
```

This milestone is limited to offline encoding, decoding, and recovery from a
malformed frame. It has no transport, hardware, or actuator interface.

## Airframe configuration subset

The second migration target is the airframe XML shape represented by
`conf/airframes/examples/bebop.xml` at the same upstream commit. The initial
Rust parser reads only the airframe name, the first firmware declaration, and
its declared targets (for example, `ap`/`bebop` and `nps`/`pc`). It does not
yet evaluate defines, modules, sections, or command laws.
