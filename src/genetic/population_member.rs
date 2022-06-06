use std::hash::Hash;

use tsp_parser::{KRandom, Tsp, TspHeuristic, TspType};

#[derive(Clone, Debug)]
pub struct PopulationMember {
    route: Vec<usize>,
    route_len: u32,
    chance: f64,
}

impl PartialEq for PopulationMember {
    fn eq(&self, other: &Self) -> bool {
        self.route_len == other.route_len
    }
}

impl Eq for PopulationMember {}

impl Hash for PopulationMember {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.route_len.hash(state);
    }
}

impl PopulationMember {
    // split into new and new_enhanced
    pub fn new_enhanced(mut route: Vec<usize>, tsp: &Tsp) -> PopulationMember {
        match tsp.get_tsp_type() {
            TspType::Symmetric => route = best_neighbourhood_invert(tsp, route),
            TspType::Asymmetric => route = best_neighbourhood_swap(tsp, route),
        }

        let route_len = tsp.get_route_len(&route).unwrap();
        let chance = 1.0 / route_len as f64;

        PopulationMember {
            route,
            route_len,
            chance,
        }
    }

    pub fn new(mut route: Vec<usize>, tsp: &Tsp) -> PopulationMember {
        let route_len = tsp.get_route_len(&route).unwrap();
        let chance = 1.0 / route_len as f64;

        PopulationMember {
            route,
            route_len,
            chance,
        }
    }

    pub fn gen_random(tsp: &Tsp) -> PopulationMember {
        let route = KRandom::new(1).get_route(tsp);

        PopulationMember::new(route, tsp)
    }

    pub fn gen_random_enhanced(tsp: &Tsp) -> PopulationMember {
        let route = KRandom::new(1).get_route(tsp);

        PopulationMember::new_enhanced(route, tsp)
    }

    pub fn get_route(&self) -> &Vec<usize> {
        &self.route
    }

    pub fn get_route_len(&self) -> u32 {
        self.route_len
    }

    pub fn get_chance(&self) -> f64 {
        self.chance
    }
}

pub fn best_neighbourhood_invert(tsp: &Tsp, route: Vec<usize>) -> Vec<usize> {
    let dimension = tsp.get_dimension();

    let mut best_route = route;
    let mut best_route_len = tsp
        .get_route_len(&best_route)
        .expect("has to be valid route");

    let mut indexes = (0, 0);
    let mut curr_best_route_len = u32::MAX;

    loop {
        for i in 0..dimension {
            for j in i + 1..dimension {
                let route_len = tsp.get_inverted_route_len(&best_route, best_route_len, i, j);

                if (route_len < best_route_len)
                    || (curr_best_route_len != u32::MAX && route_len < curr_best_route_len)
                {
                    indexes = (i, j);
                    curr_best_route_len = route_len;
                }
            }
        }

        if curr_best_route_len == u32::MAX {
            break;
        }

        let (i, j) = indexes;

        tsp_parser::neighbourhood::invert(&mut best_route[i..=j]);
        best_route_len = curr_best_route_len;

        indexes = (0, 0);
        curr_best_route_len = u32::MAX;
    }

    best_route
}

pub fn best_neighbourhood_swap(tsp: &Tsp, route: Vec<usize>) -> Vec<usize> {
    let dimension = tsp.get_dimension();

    let mut best_route = route;
    let mut best_route_len = tsp
        .get_route_len(&best_route)
        .expect("has to be valid route");

    let mut indexes = (0, 0);
    let mut curr_best_route_len = u32::MAX;

    loop {
        for i in 0..dimension {
            for j in i + 1..dimension {
                let route_len = tsp.get_swap_route_len(&best_route, best_route_len, i, j);

                if (route_len < best_route_len)
                    || (curr_best_route_len != u32::MAX && route_len < curr_best_route_len)
                {
                    indexes = (i, j);
                    curr_best_route_len = route_len;
                }
            }
        }

        if curr_best_route_len == u32::MAX {
            break;
        }

        let (i, j) = indexes;

        tsp_parser::neighbourhood::swap(&mut best_route[i..=j]);
        best_route_len = curr_best_route_len;

        indexes = (0, 0);
        curr_best_route_len = u32::MAX;
    }

    best_route
}
