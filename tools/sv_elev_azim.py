#! /usr/bin/env python3

#######################################################
# sv_elev_azim.py
# parses all NAV files
# and generates one file with (Epoch, Sv, Elev°, Azim°) 
# for testbench purposes
#######################################################
import os
import sys
import math
import georinex as gr

gr_kepler_fields = [
    "GPSWeek",
    "Toe",
    "Eccentricity",
    "sqrtA",
    "Cic",
    "Crc",
    "Cis",
    "Crs",
    "Cuc",
    "Cus",
    "DeltaN",
    "Omega0",
    "omega",
    "Io",
    "OmegaDot",
    "IDOT",
    "M0",
]

known_ref_positions = {
    "MOJN00DNK_R_20201770000_01D_MN.rnx.gz": (3628427.9118, 562059.0936, 5197872.2150),
}

def kepler_hasnan(kepler):
    for key in kepler.keys():
        if math.isnan(kepler[key]):
            return True
    return False

def main(argv):
    if len(argv) == 0:
        print("./sv_elev_azim.py [test_resources/]")
        return 0
    
    base_dir = argv[0]
    supported_rev = ["V2", "V3"]

    for rev in os.listdir(base_dir + "/NAV"):
        if not(rev in supported_rev):
            continue
        for fp in os.listdir(base_dir + "/NAV/{}".format(rev)):  
            nav_path = base_dir + "/NAV/{}/{}".format(rev, fp)
            nav = gr.load(nav_path) 

            known_ref_pos = None
            for key in known_ref_positions.keys():
                if key in nav_path:
                    known_ref_pos = known_ref_positions[key]

            txt_path = base_dir + "/gr/{}/{}.txt".format(rev, fp)
            with open(txt_path, "w") as fd:
                epochs = nav["time"].values
                vehicles = nav["sv"].values
                for epoch in epochs :
                    data = nav.sel(time=epoch)
                    for sv in vehicles:
                        sv_data = data.sel(sv=sv)
                        kepler = {}
                        for field in gr_kepler_fields:
                            value = sv_data.variables[field].values
                            kepler[field] = value
                        if not kepler_hasnan(kepler):
                            # kepler struct fully defined, can determine (Elev°,Azim°)
                            # print(epoch, sv, kepler)
                            content = "epoch, {}, ref_pos, {}, sv, {}, ".format(epoch, str(known_ref_pos), sv)
                            for key in kepler.keys():
                                content += "{}, {}, ".format(key, kepler[key])
                            fd.write(content+"\n")
    return 0

if __name__ == "__main__":
    main(sys.argv[1:])
