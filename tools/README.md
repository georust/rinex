Tools
=====

## DOY

DOY is a small python script to determine the DOY either of the current day, or a custom datetime.  
This is useful to quickly determine what the DOY of a given day was, and download data from remote FTP servers
that typically sort GNSS data by DOY.

1. Determine the DOY for today:

```bash
./tools/doy.py
```

2. Determine what was the DOY for 2023-02-28:

```bash
./tools/doy.py 2023-02-28 %Y-%m-%d
```

## Development

Set to development and testing tools.

- `builddoc.sh` builds the API doc exactly how publication with cargo does
- `testlib.sh` tests the API built with several different options
