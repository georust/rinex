Constellation:
* [ ] constellation::Geo ?    
p44   
"is almost identical to Constellation == glonass
but the records contain the satellite position, velocity, accel, health.."

General :
* [ ] add to::file production method
* [ ] simplify line interations with "for line in lines.next()"

Header:
* [ ] some (V3) files crash when parsing LEAP SECOND field 
* [ ] time of first and last obs parsing is faulty
* [ ] header.antenna.model sometimes appear as dirty, check this out
* [ ] coords [m] system ?
* [x] rcvr - clock offset applied
 * [ ] data compensation to do with this?
* [ ] GnssTime + possible conversion needed ?
* [x] interval
* [ ] missing fields: 
 * [ ] WaveLength fact L1/2 ?
 * [ ] Glonass COD/PHS/BIS ?
 * [ ] SYS / PHASE Shift
 * [ ] Glonass carriers related fields
 * [ ] Glonass SLOT /freq channel ?
 * [ ] signal strength informations
 * [ ] First and Last epoch time stamps
 * [ ] Ionospheric compensation, conversions & operations

Leap + Time modules :
* [x] move to separate modules
* [ ] leap conversion / apply methods
* [ ] gnss-time conversion method 
* [ ] gnss time related operations

Comments :
* [x] move comments to a separate structure and attach an epoch to them
* [ ] re-read about FILE MERGE comments
 * [ ] file merge comment always comprise merging timestamps ?
 * [ ] file merge comments always @ header ?

Special operations:
* [ ] Merged file: 
 * [ ] methods based on comments for merging identification
* [ ] File merging (writer)

Special methods:
* [x] interval
* [ ] merged file related methods 

Epochs:
* [ ] epoch flag mask operation & special bitmask operations
 * [ ] --> abnormal epochs masking 

Observation Data:
* [ ] rescale raw phase data exceeding F14.3 format by +/- 10E9 accordingly
* [ ] SYS PHASE Shift ?
* [ ] LLI proper identification method + usage 
* [ ] SSI proper identification method + usage 

Ci:
* [ ] OBS: if rcvr clock offsets applied: check epochs do have this field
* [ ] move src/lib.rs tests to dedicated tests/ folder 

Hatanaka:
* [ ] find some CRINEX with special epoch events (flag>2) and test them
* [ ] CRINEX 1|3 special epoch content (flag>2)
will be mishandled / corrupted if they are not only made of COMMENTS
* [ ] replace `zeros` by `intertools::fill_with()` 

Meteo Data:
* [ ] parse METEO V > 3
* [ ] Sensor Geo position (x,y,z)

Clocks Data:
* [ ] V1, V2 
* [ ] V3
* [ ] V4
