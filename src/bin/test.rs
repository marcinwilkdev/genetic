use genetyk::genetic::Genetic;
use tsp_parser::*;

fn main() {
    run_from_file("test_files/berlin52.tsp", 7542);
    println!();
    run_from_file("test_files/bier127.tsp", 118282);
    println!();
    run_from_file("test_files/ch130.tsp", 6110);
    println!();
    run_from_file("test_files/br17.atsp", 39);
    println!();
    run_from_file("test_files/ft53.atsp", 6905);
    println!();
    run_from_file("test_files/ft70.atsp", 38673);
    println!();

    // run_from_file("test_files/a280.tsp", 2579);
    // println!();
}

fn run_from_file(file_name: &str, opt: u32) {
    println!("{} {}", file_name, opt);

    let tsp = TspParser::from_file(file_name).unwrap();

    let dimension = tsp.get_dimension();

    let iterations = if file_name.contains("atsp") {
        10 * dimension
    } else {
        dimension / 2
    };

    let population_size = if file_name.contains("atsp") {
        10 * dimension
    } else {
        10 * dimension
    };

    let elites = if population_size / 2 == 0 {
        1
    } else {
        population_size / 3
    };

    let pairs = population_size / 2;
    let kid_prob = 0.9;
    let mutation_prob = 0.02;
    let stagnation_iter = iterations / 10;
    let mutation_steps = 4;

    let genetic = Genetic::new(
        opt,
        iterations,
        population_size,
        elites,
        pairs,
        kid_prob,
        mutation_prob,
        stagnation_iter,
        mutation_steps,
        4,
    );

    let route = genetic.get_route(&tsp);
    let route_len = tsp.get_route_len(&route).unwrap();

    println!("{:?}", route);
    println!("\nBest route len: {}", route_len);
}
