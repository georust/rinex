//#![feature(test)]
use rinex::{
    hatanaka::{numdiff::NumDiff, textdiff::TextDiff},
    observation::*,
    prelude::*,
    //processing::*,
    reader::BufferedReader,
    record::parse_record,
};

extern crate criterion;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, *};

use std::collections::HashMap;
use std::io::BufRead;
use std::str::FromStr;

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

fn text_decompression(textdiff: &mut TextDiff, data: &[&str]) {
    for data in data {
        let r = textdiff.decompress(data);
    }
}

fn num_decompression(numdiff: &mut NumDiff, index_reinit: usize, data: &[i64]) {
    let mut index = 0;
    for data in data {
        index += 1;
        if index % index_reinit == 0 {
            numdiff.init(3, *data).unwrap();
        } else {
            let r = numdiff.decompress(*data);
        }
    }
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
    let record = parse_record(&mut reader, header);
}

fn decompression_benchmark(c: &mut Criterion) {
    /*
     * TextDiff benchmarking
     */
    let mut group = c.benchmark_group("textdiff");

    let epochs_pool = vec![
        "KD1KVK71n2Pz6AwiBcM",
        "jhyinxzgq3v6x8zMf8",
        "CjyIN8tBzFCZcjQig8J",
        "p3AhjKQKvjWpdAda",
        "ea5dVWu4UkgzIUZp9Nj",
        "vRbeL7L4StC38JOjNNk",
        "vmatRNwwO8c__aOPLp69v",
        "odzMFRVUSI4m8g2qs84",
        "pExPLrSeNhRYcOdTOr7",
        "KEcPleNxlG9vdW4waDE",
        "ohzMcNbbZm14LRCx",
        "W7Xl6fCvkaNZIXI4ImN",
        "kknmFxgbIU7rN",
        "GimZ3zrGbok7km8D17",
        "FgD6TQwbKdRuaxk",
        "dlWSE4k0xPwJlFoDiFL",
        "9QonBABq----hjyLdP0AQ9y",
        "gYhNYkFKW4nIRM3QrhS",
        "Cgajyl3ouDzRMAhAoPD",
        "SZV2vxHJzTSRjZr9Rxa",
        "qqwDdfWdePazeaze6qdzho",
        "jxeDng8sTNHVO4skvvS",
        "xjb9KfOa73HGe4U",
        "oL8k531dLdzsbSp6qfW",
        "ufHAShfefHuOf7G0yTn",
        "QLDV0vs1H9VFBsvWuoN",
        "TsHawHBBJ6yec0TUs2z",
        "m8loLwvmC@@__ @@iW6cEwy",
        "pkqFgJdNxTM2YjZbDUJ",
        "Gf5aLDwTnekdYvIQ--@azeP10",
        "zxUKBAT432t9UFJt$$@$$ZUX",
    ];

    let flags_pool = vec![
        "1", "", "3", "", "X", "0", "", "", "p", "K", "", "", "k", "G", "F", "", "", "g", "C", "S",
        "", "", "", "", "1", "3", "", "2", "3", "", "1",
    ];

    let epochs_len = epochs_pool.len();
    let init_content = "NyV1xYEQcXyv2zzlG1A";
    let mut textdiff = TextDiff::new();
    textdiff.init(init_content);
    group.bench_function("decompression/epoch", |b| {
        b.iter(|| {
            text_decompression(&mut textdiff, &epochs_pool[0..epochs_len]);
        })
    });

    let flags_len = flags_pool.len();
    let init_content = "X";
    let mut textdiff = TextDiff::new();
    textdiff.init(init_content);
    group.bench_function("decompression/flags", |b| {
        b.iter(|| {
            text_decompression(&mut textdiff, &flags_pool[0..flags_len]);
        })
    });
    group.finish(); /* conclude textdiff group */

    /*
     * NumDiff benchmarking
     */
    let mut group = c.benchmark_group("numdiff");

    let forced_init_index = 10;
    let pool_i64 = vec![
        5918760, 92440, -240, -320, -160, -580, 360, -1380, 220, //[RE-INIT]
        -19542118, 29235, -38, 1592, -931, 645, 1001, -1038, 2198, -2679, 2804, -892,
        //[RE-INIT]
        -19542118, 29235, -38, 1592, -931, 645, 1001, -1038, 2198, -2679, 2804, -892,
        //[RE-INIT]
        -19542118, 29235, -38, 1592, -931, 645, 1001, -1038, 2198, -2679, 2804, -892,
        //[RE-INIT]
        -19542118, 29235, -38, 1592, -931, 645, 1001, -1038, 2198, -2679, 2804, -892,
        //[RE-INIT]
        -19542118, 29235, -38, 1592, -931, 645, 1001, -1038, 2198, -2679, 2804, -892,
        //[RE-INIT]
        -19542118, 29235, -38, 1592, -931, 645, 1001, -1038, 2198, -2679, 2804, -892,
        //[RE-INIT]
        -19542118, 29235, -38, 1592, -931, 645, 1001, -1038, 2198, -2679, 2804, -892,
        //[RE-INIT]
        -19542118, 29235, -38, 1592, -931, 645, 1001, -1038, 2198, -2679, 2804, -892,
        //[RE-INIT]
        -19542118, 29235, -38, 1592, -931, 645, 1001, -1038, 2198, -2679, 2804, -892,
    ];
    let mut numdiff = NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER).unwrap();
    numdiff.init(3, 25065408994).unwrap();
    group.bench_function("decompression/small", |b| {
        b.iter(|| {
            num_decompression(&mut numdiff, forced_init_index, &pool_i64[0..20]);
        })
    });
    group.bench_function("decompression/big", |b| {
        b.iter(|| {
            num_decompression(
                &mut numdiff,
                forced_init_index,
                &pool_i64[0..pool_i64.len()],
            );
        })
    });
    group.finish(); /* conclude numdiff group */
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
    decompression_benchmark(c);
    //record_parsing_benchmark(c);
    //processing_benchmark(c);
}

criterion_group!(benches, benchmark);
//name = benches;
//config = profiled();
//targets = parser_benchmark
//}
criterion_main!(benches);
