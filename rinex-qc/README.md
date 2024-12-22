RINex / GNSS QC
===============

The QC library is GNSS post processing core library.  
It is capable of answering the demanding tasks of precise navigation,
in just a few lines of code. It currently supports both RINex and optionnally SP3,
but other format may be introduced in the future.

## Workspace

A session is tied to a Workspace, defined in the Configuration script.  
When deploying and working, `QcContext` needs write access to the entire workspace.

## Deployment

`QcContext` deployment is a complex yet infaillible task.
It will only fail only internal core library major failures that we are not responible for.
If you can access the Internet daily, `QcContext` will deploy with the highest precision `ITRF93` 
frame model. If Internet access is not feasible, it will rely on lower precision offline model.

The `Qc` library uses the RUST Logger internally, it will most notably let you know
how you could "enhance" your input data.

## RINex input

Stack any supported RINex to form a complex dataset very easily:

```rust
use rinex_qc::prelude::*;

// default setup
let cfg = QcConfig::default();

// Deployment 
let mut ctx = QcContext::new()
    .unwrap_or_else(|e| panic!("ctx deployment failure: {}", e));

ctx.load_file("../test_resources/OBS/V3/DUTH0630.22O")
    .unwrap();
```

## SP3 input

When built with the `sp3` feature, SP3 data sets may be loaded as well.

```rust
use rinex_qc::prelude::*;

// default setup
let cfg = QcConfig::default();

// Deployment 
let mut ctx = QcContext::new(cfg)
    .unwrap_or_else(|e| panic!("ctx deployment failure: {}", e));

ctx.load_file("../test_resources/SP3/sio06492.sp3")
    .unwrap();
```

## Gzip files

Build the library with `flate2` feature to support gzip compressed files natively:

```rust
use rinex_qc::prelude::QcContext;

// Deployment 
let mut ctx = QcContext::new()
    .unwrap_or_else(|e| panic!("ctx deployment failure: {}", e));

ctx.load_gzip_file("../test_resources/OBS/V3/240506_glacier_station.obs.gz")
    .unwrap();
```

## Reporting

One of the major purposes of the `Qc` library, is to render a geodetic report
that will allow analyzing the superset in detail. The reported content is highly dependent
on the input context obviously. The `Qc` report will help you understand your dataset as well,
for example, it will let you know if the dataset is compatible with post processed navigation.

Report rendition is always feasible and will always work, as long as the input context is not empty.  
We currently support HTML rendering, which makes the library compatible with a web server and browser.

In the following example, we load signal observations that we can then render:

```rust
use rinex_qc::prelude::*;

// default setup
let cfg = QcConfig::default();

// Deploy
let mut ctx = QcContext::new(cfg)
    .unwrap();

ctx.load_gzip_file(
    "../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    .unwrap();

// Generate a report
let report = QcReport::new(&ctx, cfg);
let _ = report.render().into_string();
```

## Custom chapters

The `Qc` report can be enhanced with custom chapters, that only need you to provide the rendition implementation.

## Post processed navigation

The `Qc` library is able to perform the challenging task of precise navigation,
in just a few lines of code. All you need to do is provide a compatible setup.
Refer to the report summary to understand if you setup is compatible.  

In the folllowing example, we provide a BRDC navigation compatible setup

```rust
use rinex_qc::prelude::*;

// default setup
let cfg = QcConfig::default();

// deploy
let mut ctx = QcContext::new(cfg)
    .unwrap();

// stack a RINex
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// stack a BRDC RINex
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// Deploy a solver
let mut solver = ctx.nav_pvt_solver()
    .unwrap();

// Collect all solutions
while let Some(pvt) = solver.next() {

}
```

## CGGTTS tracker and solutions solver

The `Qc` library is able to perform the challenging task of precise timing resolution,
in just a few lines of code as well. Instead of deploy the `NavPvtSolver`, prefer
the `CggttsSolver` which is dedicated to CGGTTS solutions solving and implements
the special sky tracker algorithm.

Any navigation compatible setup is CGGTTS compatible by definition.
In this example, this is a BRDC navigation setup:

```rust
use rinex_qc::prelude::*;

// default setup
let cfg = QcConfig::default();

// deploy
let mut ctx = QcContext::new(cfg)
    .unwrap();

// stack a RINex
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// stack a BRDC RINex
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// Deploy a solver
let mut solver = ctx.nav_cggtts_solver()
    .unwrap();

// Collect all solutions
while let Some(track) = solver.next() {
    
}
```

If you need the best of both worlds, simply deploy both: they will evolve
and resolve at their own pace. CGGTTS is more time consuming because it
resolves for every single vehicle in sight.

## Precise Point Positioning

The `Qc` library built with `sp3` feature is compatible with the ultra demanding
PPP navigation technique. Once again, it is super simple to deploy. 

An example of PPP setup would be:

```rust
use rinex_qc::prelude::*;

// default setup that we specifically tie to SP3.
// In this context, only SP3 data points will be used.
// It is currently highly recommended to use this scenario for correct PPP
// interpretation.
let cfg = QcConfig::default()
    .with_prefered_orbit(QcPreferedOrbit::SP3);

// deploy
let mut ctx = QcContext::new(cfg)
    .unwrap();

// stack a RINex
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// stack a BRDC RINex
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// stack SP3
ctx.load_gzip_file(
    "../test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz")
    .unwrap();

// Deploy a solver: we're still using NavPvtSolver
// but the presence of SP3 changes everything
let mut solver = ctx.nav_pvt_solver()
    .unwrap();

// Collect all solutions
while let Some(track) = solver.next() {
    
}
```

## PPP ultra

The previous setup is not compatible with ultra (ultimate) PPP navigation,
because the clock data provided by the SP3 were unused. 
To change that, you need an extra parameter to your `QcConfig`:

```rust
let cfg = QcConfig::default()
    .with_prefered_orbit(QcPreferedOrbit::SP3)
    .with_prefered_clock(QcPreferedClock::SP3);

// stack SP3: this one is clock compatible
// once again, check your summary report
ctx.load_gzip_file(
    "../test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz")
    .unwrap();
```

It is more common to prefer Clock RINex for that purpose. The `Qc` library
allows that once again. Simply provide that file:

```rust
use rinex_qc::prelude::*;

// default setup that we specifically tie to SP3.
// In this context, only SP3 data points will be used.
// It is currently highly recommended to use this scenario for correct PPP
// interpretation.
let cfg = QcConfig::default()
    .with_prefered_orbit(QcPreferedOrbit::SP3)
    .with_prefered_clock(QcPreferedClock::RINex);

// deploy
let mut ctx = QcContext::new(cfg)
    .unwrap();

// stack a RINex
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// stack a BRDC RINex
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// stack SP3
ctx.load_gzip_file(
    "../test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz")
    .unwrap();

// stack a Clock RINex
ctx.load_gzip_file(
    "../test_resources/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz")
    .unwrap();

// Deploy a solver: we're still using NavPvtSolver
// but the presence of SP3 changes everything
let mut solver = ctx.nav_pvt_solver()
    .unwrap();

// Collect all solutions
while let Some(track) = solver.next() {
    
}
```

## PPP Guru

Now for all PPP Gurus out there, we're still not quite there yet.  
The stacking and exploitation of `ANTex` is work in progress.

## Precise Point Positioning + CGGTTS

When both `sp3` and `cggtts` options are active, you can deploy the ultra
demanding `PPP CGGTTS` solver, that will resolve CGGTTS tracks using the PPP technique.
All you need to do, is provide a PPP compatible setup (check your summary report) and use
the CGGTTS solver:

```rust
use rinex_qc::prelude::*;

// default setup that we specifically tie to SP3.
// In this context, only SP3 data points will be used.
// It is currently highly recommended to use this scenario for correct PPP
// interpretation.
let cfg = QcConfig::default()
    .with_prefered_orbit(QcPreferedOrbit::SP3)
    .with_prefered_clock(QcPreferedClock::RINex);

// deploy
let mut ctx = QcContext::new(cfg)
    .unwrap();

// stack a RINex
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// stack a BRDC RINex
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// stack SP3
ctx.load_gzip_file(
    "../test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz")
    .unwrap();

// stack a Clock RINex
ctx.load_gzip_file(
    "../test_resources/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz")
    .unwrap();

// Deploy a solver: we're still using the previous CGGTTS Solver
// but the presence of SP3 changes everything
let mut solver = ctx.nav_cggtts_solver()
    .unwrap();

// Collect all solutions
while let Some(track) = solver.next() {
    
}
```
