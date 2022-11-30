from rinex import * 

if __name__ == "__main__":
    crinex = Crinex()
    assert(crinex.version.major == 3)
    assert(crinex.version.minor == 0)
    
    observation = ObservationData(10.0)
    assert(observation.obs == 10.0)
    assert(observation.ssi == None)
    assert(observation.lli == None)
