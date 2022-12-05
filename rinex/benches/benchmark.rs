//#![feature(test)]
use rinex::{
    hatanaka::numdiff::NumDiff, hatanaka::textdiff::TextDiff, observation::*, prelude::*,
    reader::BufferedReader, record::parse_record,
};

extern crate criterion;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, *};

use std::collections::HashMap;
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
    let _ = Rinex::from_file(fp);
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
 * Puts record section parsing to the test
 */
fn record_parsing_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("record");

    // prepare for OBS/zegv0010.21o
    let mut header = Header::basic_obs().with_observation_fields(HeaderFields {
        crinex: None,
        codes: {
            let mut map: HashMap<Constellation, Vec<String>> = HashMap::new();
            map.insert(
                Constellation::GPS,
                vec![
                    "C1", "C2", "C5", "L1", "L2", "L5", "P1", "P2", "S1", "S2", "S5",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            );
            map.insert(
                Constellation::Glonass,
                vec![
                    "C1", "C2", "C5", "L1", "L2", "L5", "P1", "P2", "S1", "S2", "S5",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            );
            map
        },
        clock_offset_applied: false,
        dcb_compensations: Vec::new(),
        scalings: HashMap::new(),
    });
    group.bench_function("OBSv2/zegv0010.21o", |b| {
        b.iter(|| {
            record_parsing("../test_resources/OBS/V2/zegv0010.21o", &mut header);
        })
    });

    // prepare for OBS/V3/ACOR00ESP
    let mut header = Header::basic_obs().with_observation_fields(HeaderFields {
        crinex: None,
        codes: {
            let mut map: HashMap<Constellation, Vec<String>> = HashMap::new();
            map.insert(
                Constellation::GPS,
                vec![
                    "C1C", "L1C", "S1C", "C2S", "L2S", "S2S", "C2W", "L2W", "S2W", "C5Q", "L5Q",
                    "S5Q",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            );
            map.insert(
                Constellation::Glonass,
                vec![
                    "C1C", "L1C", "S1C", "C2P", "L2P", "S2P", "C2C", "L2C", "S2C", "C3Q", "L3Q",
                    "S3Q",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            );
            map.insert(
                Constellation::Galileo,
                vec![
                    "C1C", "L1C", "S1C", "C5Q", "L5Q", "S5Q", "C6C", "L6C", "S6C", "C7Q", "L7Q",
                    "S7Q", "C8Q", "L8Q", "S8Q",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            );
            map.insert(
                Constellation::BeiDou,
                vec![
                    "C2I", "L2I", "S2I", "C6I", "L6I", "S6I", "C7I", "L7I", "S7I",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            );
            map
        },
        clock_offset_applied: false,
        dcb_compensations: Vec::new(),
        scalings: HashMap::new(),
    });
    group.bench_function("OBSv3/ACOR00ESP", |b| {
        b.iter(|| {
            record_parsing(
                "../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
                &mut header,
            );
        })
    });

    //prepare for CRNX/V1/delf0010.21d
    //prepare for CRNX/V3/ESBC00DNK
    //prepare for NAV/V2/ijmu3650.21n.gz
    //prepare for NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz

    group.finish(); /* concludes record section */
}

fn benchmark(c: &mut Criterion) {
    decompression_benchmark(c);
    record_parsing_benchmark(c);
}

criterion_group!(benches, benchmark);
//name = benches;
//config = profiled();
//targets = parser_benchmark
//}
criterion_main!(benches);
