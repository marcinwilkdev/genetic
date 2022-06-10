use std::io::Write;

use tsp_parser::{run_heuristic_with_bench, HeuristicBench, Tsp, TspFileType, TspParser};

use crate::genetic::Genetic;

const NUM_PROBLEMS: usize = 10;

pub fn bench_threads() {
    for threads in 1..=5 {
        let mut file = std::fs::File::create(format!("bench_results/threads_{}", threads))
            .expect("Couldnt create bench file");

        for problem_size in 10..=100 {
            if problem_size % 2 == 0 {
                let tsps = generate_tsps_for_runtime(problem_size, TspFileType::LowerDiagRow);

                let duration = get_avg_duration(&tsps, threads);

                file.write(format!("{} {}\n", problem_size, duration).as_bytes())
                    .expect("couldn't write to file");
            }
        }
    }
}

pub fn bench_runtime() {
    let mut file =
        std::fs::File::create("bench_results/runtimes").expect("Couldnt create bench file");

    for n in 1..=100 {
        let tsps = generate_tsps_for_runtime(n, TspFileType::Euc2d);

        let duration = get_avg_duration(&tsps, 4);

        file.write(format!("{} {}\n", n, duration).as_bytes())
            .expect("couldn't write to file");
    }
}

fn get_avg_duration(tsps: &[Tsp], threads: usize) -> f64 {
    let mut duration_sum = 0;

    for tsp in tsps {
        let HeuristicBench {
            route: _,
            route_len: _,
            duration,
        } = run_heuristic_with_bench(
            &tsp,
            Genetic::new(0, 1000, 100, 5, 50, 0.9, 0.02, 100, 4, threads, 0.1), // default params
        );

        duration_sum += duration.as_millis();
    }

    let duration_avg = duration_sum as f64 / tsps.len() as f64;

    duration_avg
}

fn generate_tsps_for_runtime(size: usize, file_type: TspFileType) -> Vec<Tsp> {
    let tsp_type = match file_type {
        TspFileType::Euc2d => "euc_2d",
        TspFileType::LowerDiagRow => "lower_diag_row",
        TspFileType::FullMatrix => "full_matrix",
    };

    let mut tsps = Vec::with_capacity(NUM_PROBLEMS);

    for i in 1..=NUM_PROBLEMS {
        let tsp = TspParser::from_file(&format!("new_problems/{}-v{}-{}.tsp", tsp_type, i, size))
            .expect("File doesnt exist");

        tsps.push(tsp);
    }

    tsps
}
