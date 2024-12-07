DORIS
=====

DORIS is a special RINEX format, and is kind of the opposite to
standard Observation RINEX, in the sense that it is the DORIS satellite
that observes the ground station.

Ground stations are dessiminated throughout the world. DORIS observations
are very precise, with doppler and phase range observations down to the nano cycle
of carrier propagation.

One DORIS file describes data of a single DORIS satellite. The geodetic report
will emphasize which DORIS sallites have been loaded.

Rinex-cli being limited in its capacity to differentiate data (currently),
you should only load a single DORIS per session to not mix up the observers.
