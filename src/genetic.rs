mod genetic_state;
mod population_member;

use std::collections::HashSet;
use std::io::Write;
use std::sync::{Arc, Mutex};

use tsp_parser::{Tsp, TspHeuristic};

use genetic_state::GeneticState;
use population_member::PopulationMember;

fn prd(fx: u32, fref: u32) -> f64 {
    let fx = fx as f64;
    let fref = fref as f64;

    100.0 * (fx - fref) / fref
}

pub struct Genetic {
    opt: u32,
    iterations: usize,
    population_size: usize,
    elites_count: usize,
    pair_count: usize,
    crossing_prob: f64,
    mutation_prob: f64,
    stagnation_iter: usize,
    max_mutation_multiply: usize,
    threads: usize,
}

impl Genetic {
    pub fn new(
        opt: u32,
        iterations: usize,
        population_size: usize,
        elites_count: usize,
        pair_count: usize,
        crossing_prob: f64,
        mutation_prob: f64,
        stagnation_iter: usize,
        max_mutation_multiply: usize,
        threads: usize,
    ) -> Self {
        Genetic {
            opt,
            iterations,
            population_size,
            elites_count,
            pair_count,
            crossing_prob,
            mutation_prob,
            stagnation_iter,
            max_mutation_multiply,
            threads,
        }
    }

    fn insert_elite_begin(elites: &mut Vec<PopulationMember>, member: PopulationMember) {
        for i in 0..elites.len() {
            if elites[i].get_route_len() > member.get_route_len() {
                elites.insert(i, member);
                return;
            }
        }

        elites.push(member);
    }

    pub fn insert_elite(elites: &mut Vec<PopulationMember>, member: PopulationMember) {
        for i in 0..elites.len() {
            if elites[i].get_route_len() > member.get_route_len() {
                elites.pop();

                elites.insert(i, member);
                return;
            }
        }
    }

    // maybe some part of population fully random?
    fn initalize_genetic_state(&self, tsp: &Tsp) -> GeneticState {
        let mut population = Arc::new(Mutex::new(HashSet::with_capacity(2 * self.population_size)));
        let mut elites = Arc::new(Mutex::new(Vec::with_capacity(self.elites_count)));

        let elites_count = self.elites_count;

        let population_left = Arc::new(Mutex::new(self.population_size / 10));

        let mut threads = Vec::with_capacity(self.threads);

        for _ in 0..self.threads {
            let population_clone = Arc::clone(&population);
            let elites_clone = Arc::clone(&elites);
            let population_left_clone = Arc::clone(&population_left);

            let tsp = tsp.clone();

            threads.push(std::thread::spawn(move || loop {
                let member = PopulationMember::gen_random_enhanced(&tsp);

                let mut population_left_locked = population_left_clone.lock().unwrap();

                if *population_left_locked == 0 {
                    break;
                }

                if population_clone.lock().unwrap().insert(member.clone()) {
                    let mut elites = elites_clone.lock().unwrap();

                    if elites.len() < elites_count {
                        Genetic::insert_elite_begin(&mut elites, member);
                    } else {
                        Genetic::insert_elite(&mut elites, member);
                    }
                }

                *population_left_locked -= 1;
            }));
        }

        for thread in threads {
            thread.join();
        }

        {
            let mut population_locked = population.lock().unwrap();

            for _ in 0..self.population_size - population_locked.len() {
                let mut member = PopulationMember::gen_random(tsp);

                while !population_locked.insert(member.clone()) {
                    member = PopulationMember::gen_random(tsp);
                }

                let elites_len = elites.lock().unwrap().len();

                if elites.lock().unwrap()[elites_len - 1].get_route_len() > member.get_route_len() {
                    Genetic::insert_elite(&mut elites.lock().unwrap(), member);
                }
            }
        }

        GeneticState::new(population, elites)
    }
}

impl TspHeuristic for Genetic {
    fn get_route(&self, tsp: &Tsp) -> Vec<usize> {
        let dimension = tsp.get_dimension();

        print!("Generatic initial population... ");
        std::io::stdout().flush().unwrap();

        let mut genetic_state = self.initalize_genetic_state(tsp);

        println!("Done");

        let mut best_route_len = genetic_state.elites.lock().unwrap()[0].get_route_len();

        let mut stagnation_iter = self.stagnation_iter;
        let mut mutation_multiplier = 1;

        let mut curr_mutation_prob = self.mutation_prob;

        for i in 0..self.iterations {
            let parents = genetic_state.parents_selection(self.pair_count);

            genetic_state.cross_parents(
                &parents,
                self.crossing_prob,
                dimension,
                tsp,
                curr_mutation_prob,
                self.threads,
            );

            genetic_state.pick_population(self.population_size);

            let curr_best_route_len = genetic_state.elites.lock().unwrap()[0].get_route_len();

            if curr_best_route_len < best_route_len {
                best_route_len = curr_best_route_len;
                stagnation_iter = self.stagnation_iter;

                if mutation_multiplier > 1 {
                    mutation_multiplier = 1;
                    curr_mutation_prob = self.mutation_prob;
                }
            }

            stagnation_iter -= 1;

            if stagnation_iter == 0 {
                if mutation_multiplier < self.max_mutation_multiply {
                    mutation_multiplier += 1;

                    curr_mutation_prob *= self.mutation_prob * 100.0;
                }

                stagnation_iter = self.stagnation_iter;
            }

            if i % (self.iterations / 10) == 0 {
                let best_route_len = genetic_state.elites.lock().unwrap()[0].get_route_len();

                println!(
                    "{}\t {}\t {:.2}%",
                    i,
                    best_route_len,
                    prd(best_route_len, self.opt)
                );
            }
        }

        let elites = genetic_state.elites.lock().unwrap();

        elites[0].get_route().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tsp_parser::TspParser;

    #[test]
    fn population_generates_correctly() {
        let tsp = TspParser::from_file("test_files/berlin52.tsp").unwrap();

        let genetic = Genetic::new(7542, 1000, 100, 5, 50, 1.0, 0.02, 5000, 4, 1);

        let GeneticState {
            population, elites, ..
        } = genetic.initalize_genetic_state(&tsp);

        assert_eq!(100, population.lock().unwrap().len());
        assert_eq!(5, elites.lock().unwrap().len());
    }

    #[test]
    fn parents_generate_correctly() {
        let tsp = TspParser::from_file("test_files/berlin52.tsp").unwrap();

        let genetic = Genetic::new(7542, 1000, 100, 5, 50, 1.0, 0.02, 5000, 4, 1);

        let genetic_state = genetic.initalize_genetic_state(&tsp);

        let parents = genetic_state.parents_selection(genetic.pair_count);

        assert_eq!(genetic.pair_count, parents.len());

        for (first_parent, second_parent) in parents {
            assert_ne!(first_parent, second_parent);
        }
    }

    #[test]
    fn kid_cross_correctly() {
        let tsp = TspParser::from_file("test_files/berlin52.tsp").unwrap();

        let first_index = tsp.get_dimension() / 3;
        let second_index = first_index * 2;

        for _ in 0..10 {
            let first_parent = PopulationMember::gen_random(&tsp);
            let second_parent = PopulationMember::gen_random(&tsp);

            let kid = GeneticState::cross_kid(
                &first_parent,
                &second_parent,
                first_index,
                second_index,
                &tsp,
                0.0,
                false,
            );

            let mut cities = vec![false; tsp.get_dimension()];

            for i in 0..tsp.get_dimension() {
                cities[kid.get_route()[i]] = true;
            }

            for city in cities {
                assert!(city);
            }
        }
    }

    #[test]
    fn kid_cross_correctly_smaller() {
        let tsp = TspParser::from_file("test_files/br17.atsp").unwrap();

        let first_index = tsp.get_dimension() / 3;
        let second_index = first_index * 2;

        let first_parent = PopulationMember::gen_random(&tsp);
        let second_parent = PopulationMember::gen_random(&tsp);

        for i in 0..tsp.get_dimension() {
            if i == first_index || i == second_index {
                print!(" | ")
            };

            print!("{} ", first_parent.get_route()[i]);
        }

        println!();

        for i in 0..tsp.get_dimension() {
            if i == first_index || i == second_index {
                print!(" | ")
            };

            print!("{} ", second_parent.get_route()[i]);
        }

        let kid = GeneticState::cross_kid(
            &first_parent,
            &second_parent,
            first_index,
            second_index,
            &tsp,
            0.0,
            false,
        );

        println!();

        for i in 0..tsp.get_dimension() {
            if i == first_index || i == second_index {
                print!(" | ")
            };

            print!("{} ", kid.get_route()[i]);
        }

        let mut cities = vec![false; tsp.get_dimension()];

        for i in 0..tsp.get_dimension() {
            cities[kid.get_route()[i]] = true;
        }

        for city in cities {
            assert!(city);
        }
    }

    #[test]
    fn population_picked_correctly() {
        let tsp = TspParser::from_file("test_files/berlin52.tsp").unwrap();

        let genetic = Genetic::new(7542, 1000, 100, 5, 50, 1.0, 0.02, 5000, 4, 1);

        let mut genetic_state = genetic.initalize_genetic_state(&tsp);

        let parents = genetic_state.parents_selection(genetic.pair_count);

        genetic_state.cross_parents(
            &parents,
            genetic.crossing_prob,
            tsp.get_dimension(),
            &tsp,
            genetic.mutation_prob,
            1,
        );

        genetic_state.pick_population(genetic.population_size);

        assert_eq!(
            genetic.population_size,
            genetic_state.population.lock().unwrap().len()
        );
        assert_eq!(
            genetic.elites_count,
            genetic_state.elites.lock().unwrap().len()
        );

        for elite in &*genetic_state.elites.lock().unwrap() {
            assert!(genetic_state.population.lock().unwrap().contains(elite));
        }
    }
}
