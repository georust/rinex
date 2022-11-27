use crate::Cli;
use std::io::Write;
use crate::fops::{filename};
use rinex::{
    *,
    prelude::*,
};
use itertools::{
    max,
    Itertools,
};
use std::collections::HashMap;

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
     * header content
     */
    report.push_str("************** VERBOSE / SUMMARY REPORT *************** \n");
    report.push_str(&format!("rust-rnx - version: {}\n", env!("CARGO_PKG_VERSION")));
    report.push_str("******************************************************* \n\n");
    /*
     * proceed to ascii plot
     */
    //report.push_str(&ascii_plot(128, &rnx, nav));

    let fname = filename(cli.input_path());
    
    report.push_str("\n*********************\n");
    report.push_str(&format!("QC of RINEX  file(s) : {}\n", fname)); 
    if let Some(path) = cli.nav_path() {
        report.push_str(&format!("input RnxNAV file(s) : {}\n", filename(path))); 
    } else {
        report.push_str("input RnxNAV file(s) : None\n");
    }
    report.push_str("*********************\n\n");

    /*
     * General informations: file name, receiver and antenna infos
     */
    report.push_str(&format!("4-character ID          : {}\n", &fname[0..4])); 
    report.push_str("Receiver type           : ");
    let rcvr = match &rnx.header.rcvr {
        Some(rcvr) => format!("{} (# = {}) (fw = {})\n", rcvr.model, rcvr.sn, rcvr.firmware),
        _ => "None\n".to_string(),
    };
    report.push_str(&rcvr);
    report.push_str("Antenna type            : ");
    let antenna = match &rnx.header.rcvr_antenna {
        Some(a) => format!("{}       (# = {})\n", a.model, a.sn),
        _ => "None\n".to_string(),
    };
    report.push_str(&antenna);

    /*
     * Sampling infos
     */
    let epochs = rnx.epochs();
    let (e_0, e_n) = (epochs[0], epochs[epochs.len()-1]);
    let duration = e_n - e_0;
    let (sample_rate, _) = max(rnx.epoch_intervals())
        .expect("failed to determine epoch span");
    //let sample_rate = pretty_sample_rate!(sample_rate);
    report.push_str(&format!("Time of start of window : {}\n", e_0));
    report.push_str(&format!("Time of  end  of window : {}\n", e_n));
    report.push_str(&format!("Time line window length : {}, ticked every {}\n", duration, sample_rate));

    /*
     * Observation Data pre/general study
     *  grab # of Rx clock offsets
     *  grab # of Sv that came with at least 1 realization
     */
    let (
        epochs_with_obs, //epochs that came with at least 1 OBS
        total_sv_with_obs, //nb of SV that have at least 1 realization
        total_sv_without_obs, //nb of SV that do not have a single realization
        total_clk, //total of Rx clock offset found in this record
        time_between_reset, 
        mean_s, //averaged Sx accross all observations and epochs
    ) : (u32,usize,u32,u32,Duration,HashMap<String,(u32,f64,f64)>) = match rnx.record.as_obs() {
        Some(r) => {
            let mut total_clk = 0;
            let mut last_reset: Option<Epoch> = None;
            let mut time_between_reset = (0, Duration::default());
            let mut epochs_with_obs = 0;
            let mut total_sv_with_obs: Vec<Sv> = Vec::with_capacity(16);
            let mut mean_s: HashMap<String, (u32, f64, f64)> = HashMap::with_capacity(4);
            for ((epoch, flag), (clk_offset, svs)) in r {
                if *flag == EpochFlag::PowerFailure {
                    if let Some(prev) = last_reset {
                        time_between_reset.0 += 1;
                        time_between_reset.1 += *epoch - prev;
                    }
                    last_reset = Some(*epoch);
                }
                if clk_offset.is_some() {
                    total_clk += 1;
                }
                let mut has_obs = false;
                for (sv, observations) in svs {
                    if !total_sv_with_obs.contains(sv) {
                        total_sv_with_obs.push(*sv);
                    }
                    has_obs |= observations.len() > 0;
                    for (observation, data) in observations {
                        if is_sig_strength_obs_code!(observation) {
                            let code = &observation[..2];
                            if let Some((count, mean, _)) = mean_s.get_mut(code) {
                                *mean = (*mean * (*count as f64) + data.obs)/(*count as f64 +1.0);
                                *count = *count +1;
                            } else {
                                mean_s.insert(code.to_string(), (1, data.obs, 0.0));
                            }
                        }
                    }
                }
                if has_obs {
                    epochs_with_obs += 1;
                }
            }
            let mut total_sv_without_obs = 0;
            for sv in &rnx.space_vehicules() {
                if !total_sv_with_obs.contains(sv) {
                    total_sv_without_obs += 1;
                }
            }
            (epochs_with_obs, 
            total_sv_with_obs.len(), 
            total_sv_without_obs, 
            total_clk, 
            Duration::from_nanoseconds((time_between_reset.1.total_nanoseconds() / time_between_reset.0) as f64),
            mean_s)
        },
        _ => (0, 0, 0, 0, Duration::default(), HashMap::new()),
    };

    let (total_unhealthy_sv,total_sv_without_nav) = if let Some(nav) = nav {
        if let Some(_r) = nav.record.as_nav() {
            (1, 1)    
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    /*
     * Observation per tens of elevation angle,
     * until current mask is reached
     */
    let mut _possible_obs_angle: Vec<(f64, u32)> = Vec::with_capacity(4);
    let mut _complete_obs_angle: u32 = 0;
    let mut _deleted_obs_angle: u32 = 0;
    let mut _masked_obs_angle: u32 = 0;

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
    report.push_str(&format!("Observation interval    : {}\n", sample_rate));
    report.push_str(&format!("Total satellites w/ obs : {}\n", total_sv_with_obs)); 
    report.push_str(&format!("        {} unhealthy SV: {}\n", constell.to_3_letter_code(), total_unhealthy_sv)); 
    report.push_str(&format!("        {} SVs w/o OBS : {}\n", constell.to_3_letter_code(), total_sv_without_obs));
    report.push_str(&format!("        {} SVs w/o NAV : {}\n", constell.to_3_letter_code(), total_sv_without_nav));
    report.push_str(&format!("Rx tracking capability : unknown\n"));
    report.push_str(&format!("Poss. # of obs epochs  : {}\n", epochs.len()));
    report.push_str(&format!("Epochs w/ observations : {}\n", epochs_with_obs));
    report.push_str("Epochs repeated        :     0  (0.00%)\n");
    report.push_str("Possible obs >  0.0 deg: 0\n");
    report.push_str("Possible obs > 10.0 deg: 0\n");
    report.push_str("Complete obs > 10.0 deg: 0\n");
    report.push_str(" Deleted obs > 10.0 deg: 0\n");
    report.push_str("  Masked obs < 10.0 deg: 0\n");
    report.push_str("Obs w/ SV duplication  : 0 (within non-repeated epochs)\n");
    report.push_str("Moving average MP12    : 0.0 m\n");
    report.push_str("Moving average MP12    : 0.0 m\n");
    report.push_str("Points in MP moving avg: 50\n");

    for code in mean_s.keys().sorted() {
        let (n, mean, stddev) = mean_s.get(code)
            .unwrap();
        report.push_str(&format!("Mean {}                : {} (sd={}, n={})\n", code, mean, stddev, n));
    } 
    
    report.push_str(&format!("No. of Rx clock offsets: {}\n", total_clk));
    report.push_str(&format!("Total Rx clock drift   : {}\n ms/hour", 0.000));
    report.push_str(&format!("Avg time between resets: {}\n", time_between_reset));
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
    report.push_str(&format!("SSN {}               {}             {}    {}\n", e_0, e_n, 0.0, 0.0));
    
    /*
     * Generate summary report 
     */
    if !cli.quiet() {
        println!("{}", report);
    }
    let mut fd = std::fs::File::create(fp)
        .expect("failed to create summary report");
    write!(fd, "{}", report)
        .expect("failed to generate summary report");
}
