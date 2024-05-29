#! /usr/bin/env python3
###########################################
# Helper to determine DOY of given day
#
#  ./doy.py   returns today's DOY
#  ./doy.py 2023-05-30 "%Y-%m-%d"
###########################################
import sys
import datetime

def main(argv):
    if len(argv) == 0:
        now = datetime.datetime.now()
        print("Today is day {}".format(now.strftime("%j")))
    else:
        day = datetime.datetime.strptime(argv[0], argv[1])
        print("\"{}\" was day {}".format(argv[0], day.strftime("%j")))

if __name__ == "__main__":
    main(sys.argv[1:])
