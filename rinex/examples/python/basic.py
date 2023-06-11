from rinex import * 

# rinex::prelude basic examples,
# This example program depicts how you can interact with
# all the basic structures from the rust crate

def parser_example(fp):
    # parse a RINEX file
    rinex = Rinex(fp)
    # use header section 
    print("is_crinex: ", rinex.header.is_crinex())
    print("header : \n{:s}".format(str(rinex.header)))
    # use record section
    print(rinex.record)

def rinex_manual_constructor():
    # Manual construction example.
    # This is handy in data production contexts
    header = Header.basic_obs()
    print(header.is_crinex())

def sv_example():
    pass

def constellation_example():
    pass

def epoch_example():
    print("Epoch.system_now(): ", Epoch.system_now())

if __name__ == "__main__":
    parser_example("../test_resources/OBS/V3/DUTH0630.22O")
    epoch_example()
    sv_example()
    rinex_manual_constructor()
    constellation_example()
