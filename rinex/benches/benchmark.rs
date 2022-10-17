//#![feature(test)]
use rinex::{*,
    hatanaka::numdiff::NumDiff,
    hatanaka::textdiff::TextDiff,
};

extern crate criterion;
use criterion::{*,
    black_box, 
    criterion_group, 
    criterion_main, 
    Criterion,
    BenchmarkId,
};

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

/*
fn parser_benchmark(c: &mut Criterion) {
    let pool = vec![
        "CRNX/V1/delf0010.21d",
        "CRNX/V1/eijs0010.21d",
        "CRNX/V1/zegv0010.21d",
        "CRNX/V3/KMS300DNK_R_20221591000_01H_30S_MO.crx",
        "CRNX/V3/DOUR00BEL_R_20200130000_01D_30S_MO.crx",
        "OBS/V2/zegv0010.21o",
        "OBS/V2/npaz3350.21o",
        "NAV/V2/amel0010.21g",
        "NAV/V2/cbw10010.21n.gz",
        "NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx",
        "NAV/V3/BRDC00GOP_R_20210010000_01D_MN.rnx.gz",
        "NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx",
        "NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz",
    ];
    //let c = c.significance_level(0.01);
    let c = c.sample_size(20); // 10 measurements: default/10
    let c = c.measurement_time(core::time::Duration::from_secs(40)); // 2secs per measurement
    for fp in &pool {
        c.bench_function(fp, |b| {
            let fullpath = env!("CARGO_MANIFEST_DIR").to_owned()
                + "../test_resources/" + fp;
            b.iter(|| {
                parse_file(&fullpath)
            })
        });
    }
}*/

fn text_decompression (textdiff: &mut TextDiff, data: &[&str]) {
    for data in data {
        let r = textdiff.decompress(data);
    }
}

fn num_decompression (numdiff: &mut NumDiff, index_reinit: usize, data: &[i64]) {
    let mut index = 0;
    for data in data {
        index += 1;
        if index % index_reinit == 0 {
            numdiff.init(3, *data)
                .unwrap();
        } else {
            let r = numdiff.decompress(*data);
        }
    }
}

fn benchmark(c: &mut Criterion) {
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
        "1",
        "",
        "3",
        "",
        "X",
        "0",
        "",
        "",
        "p",
        "K",
        "",
        "",
        "k",
        "G",
        "F",
        "",
        "",
        "g",
        "C",
        "S",
        "",
        "",
        "",
        "",
        "1",
        "3",
        "",
        "2",
        "3",
        "",
        "1",
    ];
    
    let epochs_len = epochs_pool.len();
    let init_content = "NyV1xYEQcXyv2zzlG1A"; 
    let mut textdiff = TextDiff::new();
    textdiff.init(init_content);
    group.bench_function("decompression/epoch", |b| b.iter(|| {
        text_decompression(&mut textdiff, &epochs_pool[0..epochs_len]);
    }));
    
    let flags_len = flags_pool.len();
    let init_content = "X"; 
    let mut textdiff = TextDiff::new();
    textdiff.init(init_content);
    group.bench_function("decompression/flags", |b| b.iter(|| {
        text_decompression(&mut textdiff, &flags_pool[0..flags_len]);
    }));
    group.finish(); /* conclude textdiff group */

    /*
     * NumDiff benchmarking
     */
    let mut group = c.benchmark_group("numdiff");

    let forced_init_index = 10;
    let pool_i64 = vec![
        5918760, 92440, -240, -320, -160, -580, 360, -1380, 220, 
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
        //[RE-INIT]
        -19542118, 29235, -38, 1592, -931, 645, 1001, -1038, 2198, -2679, 2804, -892,
    ]; 
    let mut numdiff = NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)
        .unwrap();
    numdiff.init(3, 25065408994).unwrap(); 
    group.bench_function("decompression/small", |b| b.iter(|| {
        num_decompression(&mut numdiff, forced_init_index, &pool_i64[0..20]);
    }));
    group.bench_function("decompression/big", |b| b.iter(|| {
        num_decompression(&mut numdiff, forced_init_index, &pool_i64[0..pool_i64.len()]);
    }));
    group.finish(); /* conclude numdiff group */
}

criterion_group!(benches, benchmark);
//name = benches;
//config = profiled();
//targets = parser_benchmark
//}
criterion_main!(benches);
