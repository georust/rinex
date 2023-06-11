from rinex import * 

# rinex::prelude basic examples,
# This example program depicts how you can interact with
# all the basic structures from the rust crate

def parser_example(fp):
    pass

def rinex_manual_constructor():
    pass

def sv_example():
    pass

def constellation_example():
    pass


def epoch_example():
    print("Epoch.system_now(): ", Epoch.system_now())

if __name__ == "__main__":
    parser_example("test")
    epoch_example()
    rinex_manual_constructor()
    sv_example()
    constellation_example()
