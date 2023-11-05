# `rinex-cli` for `teqc`'ies

## Introduction

This aims to be a simple tutorial for those who wish to transition away from the unsupported and closed source
UNAVCO `teqc` tool towards the newer and more powerful open source `rinex-cli`.

<!-- It takes most of its `teqc`-xamples
from UNAVCOs guide at  https://www.unavco.org/software/data-processing/teqc/tutorial/tutorial.html -->

Although a full 1-1 replacement is not in scope, this guide should help you replace your existing scripts and take
advantage of a lot of new functionality.

## Installing rinex-cli

Prebuilt versions will eventually be supplied for direct download, but for now, you will have to build `rinex-cli` from
the source by
installing a Rust build environment like [rustup](https://rustup.rs/).
Once you have that, simply run `cargo install rinex-cli`, and this will compile, build and install the latest
published
version for you.

## Getting help

Extensive info on all parameters is available by running `rinex-cli` without parameters.

## rinex-cli translation

* File formats supported etc.

## rinex-cli editing

* Splicing etc.

## rinex-cli QC

To do Quality Check (QC) of satellite positioning data and get an HTML report, you must enable quality check mode with the `--qc` flag.

`rinex-cli --qc <filename>` where in teqc you would do `teqc +qc <filename>`

If you wish to only perform quality check, you instead set the `--qc-only` flag, skipping all other features for faster
rendition.

`rinex-cli --qc-only <filename>`

You can also pass a QC configuration file:

`rinex-cli --qc-cfg <filename>`
