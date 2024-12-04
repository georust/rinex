RINEX / GNSS QC
===============

The QC library is our core GNSS post processing library.  
It allows stacking several RINEX (any supported format) and / or SP3 for
enhanced capabilities. The main reason for this library, is that one RINEX format
is not enough to permit post processed navigation.

The Qc library generates a `QcReport` (also refered to as output product), from the input context
(also refered to, as Input Products). Currently, the geodetic report is our unique output product,
the library will not generate other data. 

The report content depends on the provided combination of input files.  
For example, providing Observation RINEX allows measurements to be plotted.  
But only stacking Navigation RINEX will allow 3D projection, navigation and similar analysis. 

The `QcReport` comprises one tab per input product (we call this the "dedicated" tab).  
Depending on the input context, the geodetic report will be enhanced, for example with ionosphere analysis.

The report is rendered in HTML, it is to this day the only format we can render. 
This means the output products of this library are compatible with a web server, and the web browser
is the standard option of the user to explore the results.

The geodetic report can be customized with extra Tabs, also refered to as extra chapters.  
This gives freendom to the user to enhance the basic report with custom information.  
The only requirement for this, is to implement the rendition Trait.

A report will always be rendered as long as your `QcContext` is not empty: at least one
file has been loaded.

## Default behavior

The Qc library targets high precision by default. This will explain the choices
we made for default features and capabilities.

## Create features

- RINEX format is supported by default and is not optional.
- The `sp3` feature allows stacking SP3 files in the `QcContext`
- `flate2` feature allows to to directly load Gzip compressed input products
- `plot` allows augmenting the geodetic report with graphs.

## Default features

SP3 support is provided by default, this means PPP is possible by default.  
`flate2` is active by default, because `gzip` compression is very common when sharing
GNSS datasets. For example, most FTP provide gzip compressed files.   
`plot` is active by default, because GNSS data is complex and very broad, text based analysis is far 
from enough to even understand the capabilities of the input products.

## RINEX analysis

Parse one or more RINEX files and render an analysis.
When built with `flate2` support, gzip compressed files can be naturally loaded:

```rust
use rinex_qc::prelude::*;

// Build a setup
// This will deploy with latest Almanac set for high performances
let mut ctx = QcContext::new()
    .unwrap();

let cfg = QcConfig::default(); // basic

let path = Path::new(
    "../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz"
);
let rinex = Rinex::from_path(&path)
    .unwrap();
ctx.load_rinex(&path, rinex);

// Generate a report
let report = QcReport::new(&ctx, cfg);
let _ = report.render().into_string();
```

## SP3 analysis

The QcReport works on any file combination and any supported input product.
The resulting report solely depends on the provided product combination.

Once again, gzip compressed files are naturally supported when built with `flate2` feature:

```rust
use rinex_qc::prelude::*;

// Build a setup
let mut ctx = QcContext::new()
    .unwrap();

let cfg = QcConfig::default(); // basic

let path = Path::new("../test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz");
let sp3 = SP3::from_path(&path)
    .unwrap();

ctx.load_sp3(&path, sp3);

// Generate a report
let report = QcReport::new(&ctx, cfg);
let _ = report.render().into_string();
```

## SP3 / NAV RINEX

When both SP3 and NAV RINEX files exist, we prefer SP3 for everything related
to Orbit states, because they provide highest accuracy. You can
force the consideration (along SP3) by using a custom `QcConfig`:

```rust
use rinex_qc::prelude::*;

// Build a setup
let mut ctx = QcContext::new()
    .unwrap();
let cfg = QcConfig::default(); // basic
```

## PPP analysis

PPP compliant contexts are made of RINEX files and SP3 files, for the same time frame.
The QcSummary report will let you know how compliant your input context is
and what may restrict performances:

```rust
use rinex_qc::prelude::*;

// basic setup
let mut ctx = QcContext::new().unwrap();
let cfg = QcConfig::default();
```

## Custom chapters

Format your custom chapters as `QcExtraPage` so you can create your own report!

```rust
use rinex_qc::prelude::*;

let mut ctx = QcContext::new().unwrap();
let cfg = QcConfig::default(); // basic setup
```

## More info

Refer to the RINEX Wiki pages hosted on Github and the tutorial scripts data base, shipped
with the RINEX library, for high level examples.
