DIFF
====

RINEX(A) - RINEX(B) differential analysis and example applications.

RINEX differential analysis is useful to permit several yet different
and exotic measurements. For example, you can use phase differentiation to compare a local clock that is spread into two separate receivers. For that particular scenario, see our [phase-clock](./phase-clock.sh) example.

RINEX differentiation applies to all observations format. 
[esbjrg-mojn](./esbjrg-mojn) is a demonstration of that.

Observations need to be made synchronously and we only differentiate identical observations (same physics, same signal and modulation).
