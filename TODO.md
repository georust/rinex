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
* [ ] rcvr-clock-offset-applied ? --> compensation ??
* [ ] GnssTime + possible conversion needed ?
* [ ] WaveLength fact L1/2 ?
* [ ] Glonass SLOT /freq channel ?
* [ ] Glonass COD/PHS/BIS ?

Record :
* [ ] ObsRecord: add clockoffsets to epoch record

Navigation Messages:
* [ ] improve batabase usage. `revision.minor` might be passed and must be used.
We should parse using the closest revision number

Observation Data:
* [x] parse OBS codes
* [x] get started with revision >= 1
* [ ] V > 2
* [ ] clock offsets
* [ ] last epoch always missed

Hatanaka:
* [ ] provide a decompression method to decompress most OBS data files

Meteo Data:
* [ ] parse METEO codes
* [ ] get started 

Clocks Data:
* [ ] TODO

