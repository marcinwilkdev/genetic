use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use rand::prelude::*;

use tsp_parser::{neighbourhood, Tsp, TspType};

use crate::genetic::population_member::PopulationMember;
use crate::genetic::Genetic;

pub struct GeneticState {
    pub population: Arc<Mutex<HashSet<PopulationMember>>>,
    pub elites: Arc<Mutex<Vec<PopulationMember>>>,
}

impl GeneticState {
    pub fn new(
        population: Arc<Mutex<HashSet<PopulationMember>>>,
        elites: Arc<Mutex<Vec<PopulationMember>>>,
    ) -> GeneticState {
        GeneticState { population, elites }
    }

    pub fn parents_selection(
        &self,
        pair_count: usize,
    ) -> Vec<(PopulationMember, PopulationMember)> {
        let mut parents = Vec::with_capacity(pair_count);

        let chances_sum = self
            .population
            .lock()
            .unwrap()
            .iter()
            .map(|p| p.get_chance())
            .sum::<f64>();

        for _ in 0..pair_count {
            let first_parent = self.select_parent(chances_sum);
            let mut second_parent = self.select_parent(chances_sum);

            while second_parent == first_parent {
                second_parent = self.select_parent(chances_sum);
            }

            parents.push((first_parent, second_parent));
        }

        parents
    }

    fn select_parent(&self, chances_sum: f64) -> PopulationMember {
        let mut random_value = thread_rng().gen::<f64>() * chances_sum;

        let curr_population = self.population.lock().unwrap();
        let mut curr_population_member = curr_population.iter().next().unwrap();

        for population_member in &*curr_population {
            random_value -= population_member.get_chance();

            if random_value < 0.0 {
                return population_member.clone();
            }

            curr_population_member = population_member;
        }

        curr_population_member.clone()
    }

    pub fn cross_parents(
        &mut self,
        parents: &[(PopulationMember, PopulationMember)],
        crossing_prob: f64,
        dimension: usize,
        tsp: &Tsp,
        mutation_prob: f64,
        threads: usize,
        memetic_fraction: f64,
    ) {
        let parents_per_thread = parents.len() / threads;
        let parents_iter = parents.chunks(parents_per_thread);

        let mut threads = Vec::with_capacity(threads);

        for pairs in parents_iter {
            let pairs_chunk = pairs.to_vec();
            let tsp = tsp.clone();

            let population_clone = Arc::clone(&self.population);
            let elites_clone = Arc::clone(&self.elites);

            threads.push(std::thread::spawn(move || {
                for i in 0..pairs_chunk.len() {
                    let pair = &pairs_chunk[i];

                    let modulus = (1.0 / memetic_fraction).round() as usize;

                    let (first_kid, second_kid) = if i % modulus == 0 {
                        Self::cross_pair(pair, crossing_prob, dimension, &tsp, mutation_prob, true)
                    } else {
                        Self::cross_pair(pair, crossing_prob, dimension, &tsp, mutation_prob, false)
                    };

                    if let Some(ref kid) = first_kid {
                        if population_clone.lock().unwrap().insert(kid.clone()) {
                            Genetic::insert_elite(&mut elites_clone.lock().unwrap(), kid.clone());
                        }
                    }

                    if let Some(ref kid) = second_kid {
                        if population_clone.lock().unwrap().insert(kid.clone()) {
                            Genetic::insert_elite(&mut elites_clone.lock().unwrap(), kid.clone());
                        }
                    }
                }
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }

    fn cross_pair(
        (first_parent, second_parent): &(PopulationMember, PopulationMember),
        crossing_prob: f64,
        dimension: usize,
        tsp: &Tsp,
        mutation_prob: f64,
        enhance: bool,
    ) -> (Option<PopulationMember>, Option<PopulationMember>) {
        let mut first_kid = None;
        let mut second_kid = None;

        let first_index = dimension / 3;
        let second_index = first_index * 2;

        if thread_rng().gen::<f64>() < crossing_prob {
            first_kid = Some(GeneticState::cross_kid(
                first_parent,
                second_parent,
                first_index,
                second_index,
                tsp,
                mutation_prob,
                enhance,
            ));
        }

        if thread_rng().gen::<f64>() < crossing_prob {
            second_kid = Some(GeneticState::cross_kid(
                second_parent,
                first_parent,
                first_index,
                second_index,
                tsp,
                mutation_prob,
                enhance,
            ));
        }

        (first_kid, second_kid)
    }

    pub fn cross_kid(
        first_parent: &PopulationMember,
        second_parent: &PopulationMember,
        first_index: usize,
        second_index: usize,
        tsp: &Tsp,
        mutation_prob: f64,
        enhance: bool,
    ) -> PopulationMember {
        let mut kid_route = first_parent.get_route().clone();

        let fragment_len = second_index - first_index;

        for i in 0..fragment_len {
            kid_route[first_index + i] = second_parent.get_route()[first_index + i];
        }

        let mut first_similarities = vec![0; tsp.get_dimension()];
        let mut second_similarities = vec![0; tsp.get_dimension()];

        for i in 0..fragment_len {
            first_similarities[first_parent.get_route()[first_index + i]] += 1;
            second_similarities[second_parent.get_route()[first_index + i]] += 1;
        }

        let mut indexes_and_cities = vec![];

        for i in 0..fragment_len {
            let city = first_parent.get_route()[first_index + i];

            if second_similarities[city] == 0 {
                for i in (0..first_index).chain(second_index..tsp.get_dimension()) {
                    if second_parent.get_route()[i] == city {
                        indexes_and_cities.push((i, city));
                        break;
                    }
                }
            }
        }

        indexes_and_cities.sort_by(|(index1, _), (index2, _)| index1.cmp(index2));

        let mut iac_iter = indexes_and_cities.iter();

        for i in 0..fragment_len {
            let city = &mut kid_route[first_index + i];

            if first_similarities[*city] == 0 {
                let &(_, swap_city) = iac_iter.next().unwrap();

                *city = swap_city;
            }
        }

        for i in 0..tsp.get_dimension() - 1 {
            let mutation_chance = mutation_prob / tsp.get_dimension() as f64;

            if mutation_chance > thread_rng().gen() {
                let second_index = thread_rng().gen_range(i + 1..tsp.get_dimension());

                match tsp.get_tsp_type() {
                    TspType::Symmetric => neighbourhood::invert(&mut kid_route[i..=second_index]),
                    TspType::Asymmetric => neighbourhood::swap(&mut kid_route[i..=second_index]),
                }
            }
        }

        if enhance {
            PopulationMember::new_enhanced(kid_route, tsp)
        } else {
            PopulationMember::new(kid_route, tsp)
        }
    }

    pub fn pick_population(&mut self, population_size: usize) {
        let mut new_population = HashSet::with_capacity(2 * population_size);

        for elite in &*self.elites.lock().unwrap() {
            new_population.insert(elite.clone());
        }

        let chances_sum = self
            .population
            .lock()
            .unwrap()
            .iter()
            .map(|p| p.get_chance())
            .sum::<f64>();

        while new_population.len() < population_size {
            new_population.insert(self.select_parent(chances_sum));
        }

        *self.population.lock().unwrap() = new_population;
    }

    pub fn gen_random_population(&mut self, population_size: usize, tsp: &Tsp) {
        let mut new_population = HashSet::with_capacity(tsp.get_dimension());
        let mut locked_elites = self.elites.lock().unwrap();

        for elite in &*locked_elites {
            new_population.insert(elite.clone());
        }

        for _ in 0..population_size - locked_elites.len() {
            let mut member = PopulationMember::gen_random(tsp);

            while !new_population.insert(member.clone()) {
                member = PopulationMember::gen_random(tsp);
            }

            if locked_elites[locked_elites.len() - 1].get_route_len() > member.get_route_len() {
                Genetic::insert_elite(&mut locked_elites, member);
            }
        }

        self.population = Arc::new(Mutex::new(new_population));
    }
}
