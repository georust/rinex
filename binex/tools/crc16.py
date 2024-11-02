#! /usr/bin/env python3

def generate_crc16_look_up_table():
    with open("output.txt", "w") as fd:
        polynomial = 0x8161
        for i in range(256):
            crc = i << 8
            for _ in range(8):
                if crc & 0x8000:
                    crc = (crc << 1) ^ polynomial
                else:
                    crc = crc << 1
           
            crc &= 0xffff
            fd.write("0x{:04X}, ".format(crc))
            if (i+1) % 8 == 0 :
                fd.write("\n")

if __name__ == "__main__":
    generate_crc16_look_up_table()
