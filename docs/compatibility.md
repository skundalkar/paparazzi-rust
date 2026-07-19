# Compatibility policy

Every implemented feature must record:

1. the upstream Paparazzi tag and commit used as reference;
2. the public interface or behavior being matched;
3. fixture/capture provenance; and
4. the comparison rule, including numeric tolerances where applicable.

Tests must prefer known input/output fixtures and differential replay against
an upstream simulator. Compatibility does not imply suitability for flight.
