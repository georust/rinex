Python3 wheel
=============

Install requirements: python3 (>3.8) and maturin 

Then build the python bindings with `maturin`

```bash
cd rinex/
maturin -F pyo3
```

A pip3 wheel is generated for your architecture. Use pip3 once again,
to install the library

```bash
pip3 --force-reinstall install rinex/target/wheels/rinex-xxx.whl
```

Now move on to the provided examples, run the basics with

```bash
python3 rinex/examples/python/basic.py
```
