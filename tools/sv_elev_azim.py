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
    "GPSWeek",
    "BDTWeek",
    "GALWeek",
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

def is_gr_kepler_key(key):
    return key in gr_kepler_keys

def is_gr_perturb_key(key):
    return key in gr_perturb_keys

def sv_is_glonass(sv):
    return sv[0] == 'R'

def sv_is_galileo(sv):
    return sv[0] == 'E'

def sv_is_gps(sv):
    return sv[0] == 'G'

def sv_is_beidou(sv):
    return sv[0] == 'C'

def sv_is_sbas(sv):
    return sv[0] == 'S'

def sv_to_constell(sv):
    if sv[0] == 'G':
        return "GPS"
    elif sv[0] == 'E':
        return "GAL"
    elif sv[0] == 'C':
        return "BDT"
    elif sv[0] == 'J':
        return "QZSS"
    else:
        return None

def timescale_t0(sv):
    if sv_is_gps(sv):
        return datetime(1980, 1, 6)
    elif sv_is_beidou(sv):
        return datetime(1980, 1, 6) # TODO
    elif sv_is_galileo(sv):
        return datetime(1980, 1, 6) # TODO
    elif sv_is_qzss(sv):
        return datetime(1980, 1, 6)
    else:
        return None #will not happen

def form_entry(fd, epoch, sv, ref_pos, ecef, elev, azi, kepler):
    fd.write("{\n")
    fd.write("  \"epoch\": \"{} UTC\",\n".format(epoch))
    fd.write("  \"sv\": {\n")
    fd.write("    \"prn\": {},\n".format(int(sv[1:])))
    fd.write("    \"constellation\": \"{}\"\n".format(sv_to_constell(sv[0])))
    fd.write("  },\n")
    fd.write("  \"week\": {},\n".format(int(kepler_weekcounter(kepler))))
    fd.write("  \"ref_pos\": [{},{},{}],\n".format(ref_pos[0], ref_pos[1], ref_pos[2]))
    fd.write("  \"ecef\": [{},{},{}],\n".format(ecef[0], ecef[1], ecef[2]))
    fd.write("  \"elev\": {},\n".format(str(elev)))
    fd.write("  \"azi\": {},\n".format(str(azi)))
    fd.write("  \"kepler\": {\n")
    fd.write("    \"a\": {},\n".format(math.pow(kepler["sqrtA"], 2)))
    fd.write("    \"e\": {},\n".format(kepler["Eccentricity"]))
    fd.write("    \"i_0\": {},\n".format(kepler["Io"]))
    fd.write("    \"omega_0\": {},\n".format(kepler["Omega0"]))
    fd.write("    \"m_0\": {},\n".format(kepler["M0"]))
    fd.write("    \"omega\": {},\n".format(kepler["omega"]))
    fd.write("    \"toe\": {}\n".format(kepler["Toe"]))
    fd.write("  },\n")
    fd.write("  \"perturbations\": {\n")
    fd.write("    \"dn\": {},\n".format(math.pow(kepler["DeltaN"], 2)))
    fd.write("    \"i_dot\": {},\n".format(kepler["IDOT"]))
    fd.write("    \"omega_dot\": {},\n".format(kepler["OmegaDot"]))
    fd.write("    \"cus\": {},\n".format(kepler["Cus"]))
    fd.write("    \"cuc\": {},\n".format(kepler["Cuc"]))
    fd.write("    \"cis\": {},\n".format(kepler["Cis"]))
    fd.write("    \"cic\": {},\n".format(kepler["Cic"]))
    fd.write("    \"crs\": {},\n".format(kepler["Crs"]))
    fd.write("    \"crc\": {}\n".format(kepler["Crc"]))
    fd.write("  }\n")
    fd.write("}")

def kepler_hasnan(kepler):
    for key in kepler.keys():
        if math.isnan(kepler[key]):
            return True
    return False

def kepler_weekcounter(kepler):
    for k in ["GPSWeek", "GALWeek", "BDTWeek"]:
        if k in kepler:
            return int(kepler[k])
    return None

def kepler_has_weekcounter(kepler):
    return kepler_weekcounter(kepler) is not None

def kepler_ready(kepler):
    if kepler_hasnan(kepler):
        return False
    if not(kepler_has_weekcounter(kepler)):
        return False
    for key in gr_kepler_fields:
        if not "Week" in key: # already tested
            if not(key in kepler):
                return False # key is missing
    return True

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
            # debug
            # print("FILE: ", nav_path) 

            nav = gr.load(nav_path) 

            ref_position = (3628427.9118, 562059.0936, 5197872.2150)

            txt_path = base_dir + "/gr/{}/{}.txt".format(rev, fp)
            with open(txt_path, "w") as fd:
                epochs = nav["time"].values
                vehicles = nav["sv"].values
                for epoch in epochs :
                    data = nav.sel(time=epoch)
                    for sv in vehicles:
                        if sv_is_glonass(sv):
                            continue # GLO: NOT YET
                        if sv_is_sbas(sv):
                            continue # GEO: NOT YET
                        if sv_to_constell(sv) is None:
                            continue # GNSS: not supported yet or unknown definition

                        sv_data = data.sel(sv=sv)

                        kepler = {}
                        for field in gr_kepler_fields:
                            # week counter special case
                            if field in sv_data:
                                kepler[field] = sv_data.variables[field].values

                        # debug 
                        # print("sv: ", sv, "kepler ready", kepler_ready(kepler)) 

                        if kepler_ready(kepler):
                            # kepler struct fully defined:
                            # we have everything to determine
                            # and space vehicle vectors, and elev° and azim °
                            weeks = kepler_weekcounter(kepler)

                            tgnss = timescale_t0(sv)
                            tgnss += timedelta(weeks=weeks)

                            # need offset within that week
                            week_offset = epoch.astype("datetime64[us]").astype(datetime)
                            week_offset -= timescale_t0(sv)
                            week_offset -= timedelta(weeks=weeks)
                            tgnss += week_offset 

                            (xref, yref, zref) = ref_position 
                            struct = xarray.Dataset(
                                kepler,
                                attrs={
                                    "svtype": sv[0], 
                                    #"xref": xref, 
                                    #"yref": yref,
                                    #"zref": zref,
                                },
                                coords={"time": [tgnss]},
                            )
    
                            expected_ecef = list(gr.keplerian2ecef(struct))
                            expected_ecef = (expected_ecef[0][0], expected_ecef[1][0], expected_ecef[2][0])

                            # content = "epoch, {}, ref_pos, {}, expected, {}, sv, {}, ".format(epoch, str(known_ref_pos), str(expected_ecef), sv)
                            #for key in kepler.keys():
                            #    content += "{}, {}, ".format(key, kepler[key])
                            form_entry(
                                fd, 
                                epoch, 
                                sv,
                                ref_position,
                                expected_ecef,
                                0.0, # elev
                                0.0, # azim
                                kepler)
                            
                            if sv == vehicles[-1]:
                                if epoch == epochs[-1]:
                                    fd.write("\n")
                                else:
                                    fd.write(",\n")
                            fd.write(",\n")
    return 0

if __name__ == "__main__":
    main(sys.argv[1:])
