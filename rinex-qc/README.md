RINEX / GNSS QC
===============

The QC library is our core GNSS post processing library.

It allows stacking GNSS data to form a complex superset that answers the requirements
of post processed navigation. The QC library currently supports RINEX and SP3 datasets,
but other formats may be introduced in the future.

## Input 

`QcContext` is the main structure. It allows stacking and indexing RINEX datasets correctly.
When built with the `sp3` feature, SP3 data sets may be loaded as well. It operates according
to the `QcConfig` structure, that allows controlling a session easily. 

## Workspace

A session will generate data into the `workspace`. The workspace location is defined
manually in the `QcConfig`uration file. The library will make sure the workspace exists
and will create it for you: all you need is write access to it.

## Reporting

A session will generate a `QcReport` (also refered to as output product). 
Its content is highly dependent on the input context. For example, you can only form
navigation solutions if your context allows post processed navigation.

Since the QC library can always generate a report, you can actually use it to
understand what your dataset actually permits (especially with `summary` reporting style). 
You can also use the session logs that will let you know how your session could be enhanced.

The report is rendered in HTML and are therefore compatible with a web server and a web browser.  
Extra chapters with custom content are allowed, it allows the user to form a complex geodetic report
with chapters that initial Qc library is not aware of. All you need to do is implement the rendition `Trait`.

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
