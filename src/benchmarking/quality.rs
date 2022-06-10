use std::fs::File;
use std::io::Write;

use crate::benchmarking;
use crate::genetic::Genetic;

use tsp_parser::*;

const NUM_ITERATIONS: usize = 10;

pub fn all_files_memetic_fraction_dependence() {
    let tsps = benchmarking::get_symmetric_problems_with_opt();

    let tsps_slice = &tsps[..14];

    for tsp in tsps_slice {
        let file_name = format!("test_files/{}.tsp", tsp.0);

        let mut file = File::create(&format!("bench_results/{}_memetic_dependence", tsp.0))
            .expect("couldnt create file");

        memetic_fraction_dependence(&mut file, &file_name, tsp.1);

        println!("{} generated.", tsp.0);
    }
}

fn memetic_fraction_dependence(file: &mut File, file_name: &str, fref: u32) {
    let tsp = TspParser::from_file(file_name).expect("file doesn't exist");
    let tsp_dimension = tsp.get_dimension();

    let iterations = if file_name.contains("atsp") {
        10 * tsp_dimension
    } else {
        tsp_dimension
    };

    let kid_prob = 0.9;
    let stagnation_iter = iterations / 5;

    let population_size = tsp_dimension;

    let elites = if population_size / 2 == 0 {
        1
    } else {
        population_size / 2
    };

    let memetic_fraction = 0.1;

    calculate_prd_and_save(
        "10%",
        &tsp,
        file,
        population_size,
        iterations,
        elites,
        kid_prob,
        stagnation_iter,
        4,
        memetic_fraction,
        fref,
    );

    let memetic_fraction = 0.2;

    calculate_prd_and_save(
        "20%",
        &tsp,
        file,
        population_size,
        iterations,
        elites,
        kid_prob,
        stagnation_iter,
        4,
        memetic_fraction,
        fref,
    );

    let memetic_fraction = 0.5;

    calculate_prd_and_save(
        "50%",
        &tsp,
        file,
        population_size,
        iterations,
        elites,
        kid_prob,
        stagnation_iter,
        4,
        memetic_fraction,
        fref,
    );
}

pub fn all_files_population_size_dependence() {
    let tsps = benchmarking::get_symmetric_problems_with_opt();

    let tsps_slice = &tsps[..14];

    for tsp in tsps_slice {
        let file_name = format!("test_files/{}.tsp", tsp.0);

        let mut file = File::create(&format!("bench_results/{}_population_dependence", tsp.0))
            .expect("couldnt create file");

        population_size_dependence(&mut file, &file_name, tsp.1);

        println!("{} generated.", tsp.0);
    }
}

fn population_size_dependence(file: &mut File, file_name: &str, fref: u32) {
    let tsp = TspParser::from_file(file_name).expect("file doesn't exist");
    let tsp_dimension = tsp.get_dimension();

    let iterations = if file_name.contains("atsp") {
        10 * tsp_dimension
    } else {
        tsp_dimension
    };

    let kid_prob = 0.9;
    let stagnation_iter = iterations / 5;

    let population_size = tsp_dimension;
    let memetic_fraction = 0.1;

    let elites = if population_size / 2 == 0 {
        1
    } else {
        population_size / 2
    };

    calculate_prd_and_save(
        "n",
        &tsp,
        file,
        population_size,
        iterations,
        elites,
        kid_prob,
        stagnation_iter,
        4,
        memetic_fraction,
        fref,
    );

    let population_size = 5 * tsp_dimension;

    let elites = if population_size / 2 == 0 {
        1
    } else {
        population_size / 2
    };

    calculate_prd_and_save(
        "5n",
        &tsp,
        file,
        population_size,
        iterations,
        elites,
        kid_prob,
        stagnation_iter,
        4,
        memetic_fraction,
        fref,
    );

    let population_size = 10 * tsp_dimension;

    let elites = if population_size / 2 == 0 {
        1
    } else {
        population_size / 2
    };

    calculate_prd_and_save(
        "10n",
        &tsp,
        file,
        population_size,
        iterations,
        elites,
        kid_prob,
        stagnation_iter,
        4,
        memetic_fraction,
        fref,
    );
}

pub fn all_files_symmetric_vs_asymmetric() {
    {
        let sym_tsps = benchmarking::get_symmetric_problems_with_opt();

        let sym_tsps_slice = &sym_tsps[..];

        let mut file =
            File::create("bench_results/results_symmetric").expect("couldnt create file");

        for tsp in sym_tsps_slice {
            let file_name = format!("test_files/{}.tsp", tsp.0);

            symmetric_vs_asymmetric(&mut file, &file_name, tsp.1);

            println!("{} generated.", tsp.0);
        }
    }

    {
        let asym_tsps = benchmarking::get_asymmetric_problems_with_opt();

        let mut file =
            File::create("bench_results/results_asymmetric").expect("couldnt create file");

        let asym_tsps_slice = &asym_tsps[..];

        for tsp in asym_tsps_slice {
            let file_name = format!("test_files/{}.atsp", tsp.0);

            symmetric_vs_asymmetric(&mut file, &file_name, tsp.1);

            println!("{} generated.", tsp.0);
        }
    }
}

fn symmetric_vs_asymmetric(file: &mut File, file_name: &str, fref: u32) {
    let tsp = TspParser::from_file(file_name).expect("file doesn't exist");
    let tsp_dimension = tsp.get_dimension();

    let iterations = if file_name.contains("atsp") {
        10 * tsp_dimension
    } else {
        tsp_dimension
    };

    let population_size = 10 * tsp_dimension;

    let elites = if population_size / 2 == 0 {
        1
    } else {
        population_size / 2
    };

    let kid_prob = 0.9;
    let stagnation_iter = iterations / 5;
    let memetic_fraction = 0.1;

    calculate_prd_and_save(
        &format!("{}", tsp_dimension),
        &tsp,
        file,
        population_size,
        iterations,
        elites,
        kid_prob,
        stagnation_iter,
        4,
        memetic_fraction,
        fref,
    );
}

fn calculate_prd_and_save(
    label: &str,
    tsp: &Tsp,
    file: &mut File,
    population_size: usize,
    iterations: usize,
    elites_count: usize,
    crossing_prob: f64,
    stagnation: usize,
    threads: usize,
    memetic_fraction: f64,
    fref: u32,
) {
    let mut route_lens = Vec::with_capacity(NUM_ITERATIONS);

    for _ in 0..NUM_ITERATIONS {
        let HeuristicBench {
            route: _,
            route_len,
            duration: _,
        } = run_heuristic_with_bench(
            &tsp,
            Genetic::new(
                0,
                iterations,
                population_size,
                elites_count,
                population_size / 2,
                crossing_prob,
                0.02,
                stagnation,
                4,
                threads,
                memetic_fraction,
            ),
        );

        route_lens.push(route_len);
    }

    let route_len = route_lens.iter().sum::<u32>() / NUM_ITERATIONS as u32;

    let prd_value = prd(route_len, fref);

    file.write(format!("{} {}\n", label, prd_value).as_bytes())
        .expect("couldn't write to file");
}

fn prd(fx: u32, fref: u32) -> f64 {
    let fx = fx as f64;
    let fref = fref as f64;

    100.0 * (fx - fref) / fref
}
