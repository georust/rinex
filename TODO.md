Constellation:
* [ ] constellation::Geo ?    
p44   
"is almost identical to Constellation == glonass
but the records contain the satellite position, velocity, accel, health.."

Doc:
* [ ] epoch: determine smallest epoch for a given constellation or sv
* [ ] epoch: determine longest epoch for a given constellation or sv
* [ ] epoch: time spanning

Parsing :
* [ ] cleanup split(head, body) ?   
for some file with start the rinex body with "\n xxxxxx"
causing some issues when identifying 1st epoch.
* [x] last epoch seems to always be missed

Header:
* [ ] time of first and last obs parsing is faulty
* [ ] header.antenna.model sometimes appear as dirty, check this out
* [ ] coords [m] system ?
* [x] rcvr - clock offset applied
 * [ ] data compensation to do with this?
 * [x] simplify: set to simple boolean TRUE/FALSE
* [ ] GnssTime + possible conversion needed ?
* [ ] WaveLength fact L1/2 ?
* [ ] Glonass SLOT /freq channel ?
* [ ] Glonass COD/PHS/BIS ?
* [ ] interval ?

Record :
* [x] ObsRecord : add clockoffsets to epoch record
* [x] ObsRecord : introduce Observation(f32,lli,ssi) as payload

Navigation Messages:
* [ ] improve database usage.   
`revision.minor` might be passed and must be used.   
We should parse using the closest revision number

Observation Data:
* [x] parse OBS codes V < 3
* [x] parse OBS codes V > 2
* [x] parse OBS record V < 3
* [x] parse OBS record V > 2
* [x] parse clock offsets and classify them properly
* [ ] rescale raw phase data exceeding F14.3 format by +/- 10E9 accordingly
* [ ] SYS PHASE Shift ?

Ci:
* [ ] OBS: if rcvr clock offsets applied: check epochs do have this field

Hatanaka:
* [ ] provide a decompression method to decompress most OBS data files

Meteo Data:
* [x] parse METEO codes
* [x] parse METEO V < 3
* [x] parse METEO V > 2
* [ ] Sensor Geo position (x,y,z)

Clocks Data:
* [ ] TODO 
