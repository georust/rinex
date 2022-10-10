//#![feature(test)]
extern crate criterion;
use rinex::*;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

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
}

criterion_group!(benches, parser_benchmark);
//name = benches;
//config = profiled();
//targets = parser_benchmark
//}
criterion_main!(benches);
