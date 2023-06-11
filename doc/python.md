Python3 wheel
=============

Install requirements: python3 (>3.8) and maturin 

Then build the python bindings: use `maturin` to build the crate
with the `pyo3` feature

```bash
cd rinex/
maturin build -F pyo3
```

A pip3 wheel is generated for your architecture.   
Use pip3 to install the library

```bash
pip3 install --force-reinstall rinex/target/wheels/rinex-xxx.whl
```

Now move on to the provided examples, run the basics with

```bash
python3 rinex/examples/python/basic.py
```
