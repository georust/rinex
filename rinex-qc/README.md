RINEX / GNSS QC
===============

The QC library was created to analyze complex GNSS datasets.  
It currently accepts RINEX (all supported formats) and/or SP3 files, which are the
basic requirements to precise navigation.

The Qc library generates a `QcReport` (also refered to as output product), from the input context.
The report content depends on the provided combination of input files (also refered
to as, input products). 
QC standing for Quality Control, as it is a widely spread term in preprocessing
applications, the QC may apply to navigation applications, atmosphere analysis
and timing applications.

The `QcReport` comprises one tab per input product (dedicated tab),
may have tabs depending on the operations that the input context allows.
For example SP3 and/or BRDC RINEX will enable the `Orbit Projection tab`.

The report is render in HTML and that is currently the only format we can render.

`QcReport` allows customization with extra chapters, so you can append
as many chapters as you need, depending on your requirements and capabilities,
as long as you can implement the rendition Trait.

## Create features

- activate the `sp3` feature to support SP3 format
- activate the `plot` feature for your reports to integrate graphs analysis
- activate the `flate2` feature to directly load Gzip compressed input products

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

let mut ctx = QcContext::default();
let cfg = QcConfig::default(); // basic setup
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
