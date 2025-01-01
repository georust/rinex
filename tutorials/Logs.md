Logs and debug traces
=====================

All the ecosystem uses the Rust logger. The debug traces are therefore
controled via the `$RUST_LOG` environment variable.

Many sensitivity exist, for example `export RUST_LOG=info` will enable
the default sensitivity.

If you specify `export RUST_LOG=trace`, you will be able to see every single debug trace.

`rinex-cli`
===========

It might prove vital to understand what the toolbox tells you, especially:

- when loading complex datasets
- when navigating

## Deploy time

When the application deploys, it will let youn know whether the core dependencies are being updated
or rely on local storage. Daily internet access is required for uttermost precision.

```bash
RUST_LOG=trace \
    ./target/release/rinex-cli \
        --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz -q
```

The application has already been used today: here we rely on previously stored core dependencies

```bash
[2025-01-01T18:10:28Z DEBUG rinex_qc::context] (anise) from local storage
[2025-01-01T18:10:28Z DEBUG anise::almanac::metaload::metafile] parsing /home/guillaume/.local/share/nyx-space/anise/de440s.bsp caused relative URL without a base -- assuming local path
[2025-01-01T18:10:28Z DEBUG anise::almanac::metaload::metafile] parsing /home/guillaume/.local/share/nyx-space/anise/pck11.pca caused relative URL without a base -- assuming local path
[2025-01-01T18:10:28Z DEBUG anise::almanac::metaload::metafile] parsing /home/guillaume/.local/share/nyx-space/anise/earth_latest_high_prec.bpc caused relative URL without a base -- assuming local path
```

Most of the ecosystem uses the logger, so you may get information coming from core dependencies.
This is particularly true for Nyx, ANISE and RTK-rs: 

```bash
# ANISE deployment
[2025-01-01T18:10:28Z INFO  anise::almanac] Loading almanac from /home/guillaume/.local/share/nyx-space/anise/de440s.bsp
[2025-01-01T18:10:28Z INFO  anise::almanac] Loading as DAF/SPK
[2025-01-01T18:10:28Z INFO  anise::almanac] Loading almanac from /home/guillaume/.local/share/nyx-space/anise/pck11.pca
[2025-01-01T18:10:28Z TRACE anise::structure::dataset] [try_from_bytes] loaded context successfully
[2025-01-01T18:10:28Z INFO  anise::almanac] Loading almanac from /home/guillaume/.local/share/nyx-space/anise/earth_latest_high_prec.bpc
[2025-01-01T18:10:28Z INFO  anise::almanac] Loading as DAF/PCK
```

The most important aspect when deploying the application, is to obtain a reference frame model.  
PPP navigation is always ground based and currently limited to Earth. So we're interested in gathering 
an Earth Centered Reference frame model (aka ECEF). 

When internet access is obtained daily, you can rely on the highest precision setup
for all your sessions for that day:

```
[2025-01-01T18:10:28Z INFO  rinex_qc::context] earth_itrf93 frame model loaded
```

When internet access is not feasible for today, you can only rely on the offline model.  
The difference between the two has no meaningful impact until the uttermost precision
is targeted, possibly in very long static navigation.

For each file loaded in the context, you will get indications about its content and how it has been indexed.
It is very important to make sure your data is correctly indexed if you are interested in differential operations,
for eample RTK. If that is not the case, it does not matter, you most likely will only load a single entry
per file format.

```bash
# In the previous example, a single observation RINEx file was loaded.
# It is indexed by the GNSS receiver model, which is the default preference for this kind.
[2025-01-01T18:10:29Z DEBUG rinex_qc::context::obs] ESBC00DNK designated by SEPT POLARX5-3047937 (prefered method)
[2025-01-01T18:10:29Z INFO  rinex_qc::context] ESBC00DNK_R_20201770000_01D_30S_MO.crx (RINEx) loaded
[2025-01-01T18:10:29Z DEBUG rinex_cli] Observation RINEx: ESBC00DNK
```

### PPP navigation logs

When navigating, it is vital to unlock the debug traces to see the actual process and possibly monitor it.
