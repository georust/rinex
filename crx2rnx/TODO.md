Decompression : 
* [ ] CRX2RNX V1, V2
* [ ] add clock offset correctly (V1,V2)
* [x] CRX2RNX V3, V4
* [x] add clock offset correctly (V3,V4)
* [x] must take epoch flag into account
* [x] epoch flag > 2 : leave epoch as is
* [ ] test RNX2CRX from my CRX2RNX output 
* [ ] possibility to unzip if needed ? 
* [ ] parsing after CRX2RNX should work all the time

Compression :
* [ ] RNX2CRX V1, V2
* [ ] RNX2CRX V3, V4
* [ ] possibility to zip if desired ?

Other :
* [x] improve doc
* [x] improve definition : -d -c are mandatory but mutually exclusive
* [x] CI: check at least all command line args
* [x] could use "diff .RNX RNX. -s -q "Files [...] are identical"
