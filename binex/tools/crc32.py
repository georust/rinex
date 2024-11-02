#! /usr/bin/env python3

def generate_crc32_look_up_table():
    with open("output.txt", "w") as fd:
        polynomial = 0x4C11B7
        for i in range(256):
            crc = i << 24
            for _ in range(8):
                if crc & 0x80000000:
                    crc = (crc << 1) ^ polynomial
                else:
                    crc = crc << 1
           
            crc &= 0xffffffff
            fd.write("0x{:04X}, ".format(crc))
            if (i+1) % 8 == 0 :
                fd.write("\n")

if __name__ == "__main__":
    generate_crc32_look_up_table()
