//#![feature(test)]
use rinex::{
    prelude::*,
    //processing::*,
    reader::BufferedReader,
    record::parse_record,
};

extern crate criterion;
use criterion::{criterion_group, criterion_main, Criterion};

use std::io::BufRead;

/*struct CpuProfiler;

impl criterion::profiler::Profiler for CpuProfiler {
    fn start_profiling(&mut self, benchmark_id: &str, benchmark_dir: &Path) {
        cpuprofiler::PROFILER
            .lock()
            .unwrap()
            .start(format!("./{}.profile", benchmark_id).as_bytes())
            .unwrap();
    }

    fn stop_profiling(&mut self, benchmark_id: &str, benchmark_dir: &Path) {
        cpuprofiler::PROFILER.lock().unwrap().stop().unwrap();
    }
}

fn profiled() -> Criterion {
    Criterion::default().with_profiler(CpuProfiler)
}*/

fn parse_file(fp: &str) {
    let _ = Rinex::from_file(fp).unwrap();
}

/*
 * Browses and skip header fields
 * used in record focused benchmark
 */
fn browse_skip_header_section(reader: &mut BufferedReader) {
    let lines = reader.lines();
    for line in lines {
        let line = line.unwrap();
        if line.contains("END OF HEADER") {
            return;
        }
    }
}

/*
 * Puts record section parsing to test
 */
fn record_parsing(path: &str, header: &mut Header) {
    let mut reader = BufferedReader::new(path).unwrap();
    browse_skip_header_section(&mut reader);
    let _record = parse_record(&mut reader, header);
}

/*
 * Evaluates parsing performance of plain RINEX parsing
fn record_parsing_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources");
    /*
     * small, medium, large compressed: OBS
     */
    for (rev, filename) in vec![
        ("V2", "del0010.21o"),
    ] {
        group.bench_function("OBSv2/zegv0010.21o", |b| {
            b.iter(|| {
                record_parsing("../test_resources/OBS/V2/zegv0010.21o", &mut header);
            })
        });
    }
    group.finish(); /* concludes record section */
}
 */

//fn processing_benchmark(c: &mut Criterion) {
//    let mut group = c.benchmark_group("processing");
//    let rinex =
//        Rinex::from_file("../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
//            .unwrap();
//    let record = rinex.record.as_obs().unwrap();
//
//    for filter in vec![
//        (Filter::from_str("mask:GPS,GLO,BDS").unwrap(), "mask:gnss"),
//        //(Filter::from_str("mask:gt:10 minutes").unwrap(), "mask:dt"),
//        (
//            Filter::from_str("mask:L1C,C1C,L2P,L2W").unwrap(),
//            "mask:obs",
//        ),
//        (
//            Filter::from_str("mask:g08,g15,g19,r03,r09").unwrap(),
//            "mask:sv",
//        ),
//        //(Filter::from_str("mask:2020-06-25 08:00:00UTC").unwrap(), "mask:epoch"),
//        (Filter::from_str("smooth:hatch").unwrap(), "smoothing:hatch"),
//        (
//            Filter::from_str("smooth:hatch:l1c,l2c").unwrap(),
//            "smoothing:hatch:l1c,l2c",
//        ),
//        //(Filter::from_str("smooth:mov:10 minutes").unwrap(), "smoothing:mov:10 mins"),
//    ] {
//        let (filter, name) = filter;
//        group.bench_function(&format!("esbc00dnk_r_2021/{}", name), |b| {
//            b.iter(|| record.filter(filter.clone()))
//        });
//    }
//
//    for combination in vec![
//        (Combination::GeometryFree, "gf"),
//        (Combination::NarrowLane, "nl"),
//        (Combination::WideLane, "wl"),
//        (Combination::MelbourneWubbena, "mw"),
//    ] {
//        let (combination, name) = combination;
//        group.bench_function(&format!("esbc00dnk_r_2021/{}", name), |b| {
//            b.iter(|| {
//                record.combine(combination);
//            })
//        });
//    }
//    group.bench_function("esbc00dnk_r_2021/dcb", |b| {
//        b.iter(|| {
//            record.dcb();
//        })
//    });
//    group.bench_function("esbc00dnk_r_2021/ionod", |b| {
//        b.iter(|| {
//            record.iono_delay_detector(Duration::from_seconds(30.0));
//        })
//    });
//    group.bench_function("esbc00dnk_r_2021/derivative", |b| {
//        b.iter(|| {
//            let der = record.derivative();
//            let mov = der.moving_average(Duration::from_seconds(600.0), None);
//        })
//    });
//}

fn benchmark(c: &mut Criterion) {
    //record_parsing_benchmark(c);
    //processing_benchmark(c);
}

criterion_group!(benches, benchmark);
//name = benches;
//config = profiled();
//targets = parser_benchmark
//}
criterion_main!(benches);
