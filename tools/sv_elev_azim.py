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
import xarray
import georinex as gr
from datetime import datetime, timedelta

gr_kepler_fields = [
    "Week",
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

def kepler_has_weekcounter(kepler):
    for wk in ["GPSWeek", "GALWeek", "BDTWeek"]:
        if wk in kepler:
            return True
    return False

def kepler_ready(kepler):
    if kepler_hasnan(kepler):
        return False
    if not(kepler_has_weekcounter(kepler)):
        return False
    for key in gr_kepler_fields:
        if key != "Week": # week counter..
            if not(key in kepler):
                return False # key is missing
    return True

def sv_is_glonass(sv):
    return sv[0] == 'R'

def sv_to_constell(sv):
    if sv[0] == 'G':
        return "GPS"
    elif sv[0] == 'E':
        return "GAL"
    elif sv[0] == 'C':
        return "BDT"
    elif sv[0] == 'J':
        return "GPS"
    else:
        return None

def constell_t0(sv):
    #if sv[0] == 'G':
    return datetime(1980, 1, 6)

def main(argv):
    if len(argv) == 0:
        print("[test_pool_dir]")
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
            if known_ref_pos is None:
                known_ref_pos = (-5.67841101e6, -2.49239629e7, 7.05651887e6)

            txt_path = base_dir + "/gr/{}/{}.txt".format(rev, fp)
            with open(txt_path, "w") as fd:
                epochs = nav["time"].values
                vehicles = nav["sv"].values
                for epoch in epochs :
                    data = nav.sel(time=epoch)
                    for sv in vehicles:
                        if sv_is_glonass(sv):
                            continue # GLO: NOT YET
                        if sv_to_constell(sv) is None:
                            continue # GNSS: not supported yet or unknown definition

                        sv_data = data.sel(sv=sv)
                        kepler = {}
                        for field in gr_kepler_fields:
                            if field == "Week":
                                # week counter special case
                                field = sv_to_constell(sv) + field 
                            value = sv_data.variables[field].values
                            kepler[field] = value
                        if not kepler_hasnan(kepler):
                            # kepler struct fully defined, can determine (Elev°,Azim°)
                            # print(epoch, sv, "READY: ", kepler_ready(kepler), kepler)
                            
                            tgnss = constell_t0(sv)
                            #tgnss += timedelta(weeks=kepler[sv_to_constell(sv) + "Week"])
                            #tgnss+= epoch - previous_sunday.midnight in seconds

                            (xref, yref, zref) = known_ref_pos
                            struct = xarray.Dataset(
                                kepler,
                                attrs={
                                    "svtype": "G", 
                                    "xref": xref, 
                                    "yref": yref,
                                    "zref": zref,
                                },
                                coords={"time": [tgnss]},
                            )
    
                            expected_ecef = list(gr.keplerian2ecef(struct))
                            expected_ecef = (expected_ecef[0][0], expected_ecef[1][0], expected_ecef[2][0])

                            content = "epoch, {}, ref_pos, {}, expected, {}, sv, {}, ".format(epoch, str(known_ref_pos), str(expected_ecef), sv)
                            for key in kepler.keys():
                                content += "{}, {}, ".format(key, kepler[key])
                            fd.write(content+"\n")
    return 0

if __name__ == "__main__":
    main(sys.argv[1:])
