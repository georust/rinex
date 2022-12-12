use crate::fops::filename;
use crate::Cli;
use itertools::{max, Itertools};
use rinex::{prelude::*, *};
use std::collections::HashMap;
use std::io::Write;

//mod ascii_plot;

/// generates `teqc` summary report
/// fp: report absolute path
/// rnx: rnx (observation) to analyze
/// nav: optionnal but seriously recommended context
pub fn summary_report(cli: &Cli, prefix: &str, rnx: &Rinex, nav: &Option<Rinex>) {
    /*
     * We split reporting into every encountered GNSS constellation,
     * and we do not limit ourselves to GPS/GLO
     */
    for constell in rnx.list_constellations() {
        let path = format!("{}/{}-summary.txt", prefix, constell.to_3_letter_code());
        if !cli.quiet() {
            println!("Reporting \"{}\"..\n", path);
        }
        do_report(
            cli,
            &path,
            constell,
            /*
                by filtering what we pass here,
                the high level information, especially sample rate related,
                are naturally adjusted and relevant
            */
            &rnx.retain_constellation(vec![constell]),
            &nav,
        );
    }
}

/// Single constellation reporting
pub fn do_report(cli: &Cli, fp: &str, constell: Constellation, rnx: &Rinex, nav: &Option<Rinex>) {
    let mut report = String::new();
    /*
     * Observation per tens of elevation angle,
     * until current mask is reached
     */
    /*
     * RCVR antenna positionning
     * based of Pseudo Range observations
     */
    report.push_str(" mean antenna; # of pos :     0\n");
    report.push_str("  antenna WGS 84 (xyz)  :  0.0 0.0 0.0 (m)\n");
    report.push_str("  antenna WGS 84 (geo)  :  0     0\n");
    report.push_str("  antenna WGS 84 (geo)  :  0.0 ddeg  0.0 ddeg\n");
    report.push_str("          WGS 84 height :  0.0 m\n");
    report.push_str("|qc - header| position  :  0.0 m\n");
    report.push_str("qc position offsets     :  0.0 m vertical  0.0 m horizontal\n");
    
    report.push_str("Obs w/ SV duplication  : 0 (within non-repeated epochs)\n");
    report.push_str("Moving average MP12    : 0.0 m\n");
    report.push_str("Moving average MP12    : 0.0 m\n");
    report.push_str("Points in MP moving avg: 50\n");

    report.push_str("Freq no. and timecode   : ?\n");
    report.push_str("Report gap > than       : 10.00 minute(s)\n");
    report.push_str("epochs w/ msec clk slip : 0\n");
    report.push_str("other msec mp events    : 0 (: 234)   {expect ~= 1:50}\n");
    report.push_str("IOD signifying a slip   : >400.0 cm/minute\n");
    report.push_str("IOD slips < 10.0 deg*   :     00\n");
    report.push_str("IOD slips > 10.0 deg    :     00\n");
    report.push_str("IOD or MP slips < 10.0* :     00\n");
    report.push_str("IOD or MP slips > 10.0  :     00\n");
    report.push_str(" * or unknown elevation\n");
    report.push_str("      first epoch    last epoch     sn1   sn2\n");
}
