Constellation:
* [ ] constellation::Geo ?    
p44   
"is almost identical to Constellation == glonass
but the records contain the satellite position, velocity, accel, health.."

Doc:
* [ ] epoch: determine smallest epoch for a given constellation or sv
* [ ] epoch: determine longest epoch for a given constellation or sv

Header:
* [ ] time of first and last obs parsing is faulty
* [ ] header.antenna.model sometimes appear as dirty, check this out
* [ ] coords [m] system ?
* [ ] rcvr_clock_offset_applied ? --> compensation ??
* [ ] GnssTime + possible conversion needed ?
* [ ] WaveLength fact L1/2 ?

Record :
* [ ] ObsRecord: add clockoffsets to epoch record

Navigation Messages:
* [ ] improve batabase usage. `revision.minor` might be passed and must be used.
We should parse using the closest revision number

Observation Data:
* [ ] parse OBS codes
* [ ] get started with revision > 1

Hatanaka:
* [ ] provide a decompression method to decompress most OBS data files

Meteo Data:
* [ ] parse METEO codes
* [ ] get started 

Clocks Data:
* [ ] TODO

