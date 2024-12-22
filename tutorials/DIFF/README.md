DIFF
====

`diff` is a differential operation that applies to either Observation,
Meteo or DORIS RINex. It applies equation (a-b) where b is considered "reference",
on identical observations. We only differentiate the same observation:

- observations must be "synchronous" ie., reported as sampled at the same "instant"
by the GNSS receiver or meteo sensor
- same signal source (SV)
- same physics: phase is differenced with phase
- same modulations: L1 is substracted to L1.

The differential operation may allow several exotic measurements.
For example, substracting phase observed by two GNSS receivers that share a common clock,
may serve as a clock measurement system.

Like any other file operation, `diff` supports all file synthesis options,
including export to CSV.

Examples:
 - `phase-clock.sh` substracts phase measurements observed by two GNSS receivers
 that share a common clock
 - `esbjrg-mojn` asynchronous close range stations comparison
