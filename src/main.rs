

mod tsp;
mod setup;

extern crate cue;
extern crate rand;
extern crate num_cpus;

#[macro_use]
extern crate clap;
extern crate bidir_map;

use rand::{SeedableRng, Rng, Isaac64Rng};
use tsp::{DistMap};

// const NUM_THREADS : usize = 4;
// const POP_SIZE : usize = 50;
// const GENERATIONS : usize = 100;
// const ERAS : usize = 10;

struct Creature {
    cities : Vec<u32>
}

impl Clone for Creature {
    fn clone(&self) -> Self{
        let mut x : Vec<u32> = Vec::new();
        for c in self.cities.iter() {
            x.push(*c);
        }
        Creature{cities : x}
    }
}

impl Creature {
    fn new_random<R : Rng>(r : &mut R, num_cities : u32) -> Creature {
        let mut v = Vec::new();
        for x in 0..num_cities {
            v.push(x);
        }
        r.shuffle(&mut v);
        Creature {cities : v}
    }

    fn mutate<R : Rng>(&mut self, r : &mut R, num_cities : u32) {
        let ind_a = (r.gen::<u32>() % num_cities) as usize;
        let ind_b = (r.gen::<u32>() % num_cities) as usize;
        self.cities.swap(ind_a, ind_b);
    }

    fn breed_with<R : Rng>(&self, other : &Creature, r : &mut R, num_cities : u32) -> Creature {
        let offset = r.gen::<u32>() % num_cities;
        let mut v = Vec::new();
        let copylen = (num_cities as f32 * r.gen::<f32>()) as usize;
        for i in 0..copylen {
            let index = (i + offset as usize) % num_cities as usize;
            v.push(self.cities[index]);
        };
        for x in other.cities.iter() {
            if ! v.contains(x) {
                v.push(*x);
            }
        }
        Creature {cities : v}
    }

    fn obj_func(&self, dmap : &DistMap) -> f32 {
        let mut tot = 0.0;
        let mut prev = *self.cities.last().unwrap();
        for &city in self.cities[0..].iter() {
            if let Some(cost) = dmap.get(&(prev, city)) {
                tot += *cost;
            } else {
                return std::f32::MAX;
            }
            prev = city;
        }
        tot
    }
}

// fn prep() {
//     let (cities, points) = tsp::read_point_map("./tsp_points").unwrap();
//     let distances = tsp::point_map_to_dist_map(&cities, &points);
//     tsp::write_dist_map(&cities, &distances, "./out.txt").unwrap();
// }

fn main() {

    let config = setup::configure();
    println!("{:#?}", &config);
    // prep();



    let (cities, distances) = if config.in_mode == setup::InMode::DistMode {
        tsp::read_dist_map(&config.in_path).unwrap()
    } else {
        let (c, points) = tsp::read_point_map(&config.in_path).unwrap();
        let d = tsp::point_map_to_dist_map(&c, &points);
        (c, d)
    };

    if config.dist_path != ""{
        tsp::write_dist_map(&cities, &distances, &config.dist_path).is_ok();
    }
    let num_cities : u32 = cities.len() as u32;

    let mut rng = Isaac64Rng::from_seed(&[1,2,3,4,5,6,8]);

    let mut populations : Vec<Vec<Creature>> = Vec::new();
    for _ in 0..config.worker_threads {
        populations.push(fresh_group(config.population, &mut rng, num_cities));
    }

    for era in 0..config.eras {
        let mut next_populations : Vec<Vec<Creature>>  = Vec::new();
        cue::pipeline(
            "idk",
            config.worker_threads,
            populations.iter(),
            |input| {
                evolve(input, rng, &distances, num_cities, &config)
            },
            |output| {
                next_populations.push(output)
            },
        );
        // for z in populations.iter() {
        //     let q = evolve(z, rng, &distances, num_cities, &config);
        //     next_populations.push(q);
        // }

        let mut next_gen : Vec<Creature> = Vec::new();
        for p in next_populations {
            for z in p {
                next_gen.push(z);
            }
        }
        sort_pop_objectively(&mut next_gen, &distances);
        next_gen.dedup_by(
            |a, b|
            a.cities.cmp(& b.cities) == Ordering::Equal
        );
        while next_gen.len() > config.population {
            next_gen.pop();
        }
        while next_gen.len() < config.population {
            next_gen.push(Creature::new_random(&mut rng, num_cities));
        }

        println!("Best of Era {}/{}\thas {}", era+1, config.eras, &next_gen[0].obj_func(&distances));
        if era == config.eras-1 {
            println!("\n\n");
            for c in next_gen[0].cities.iter(){
                print!("{} --> ", cities.get_by_first(c).unwrap());
            }
            break;
        }
        populations = Vec::new();
        for _ in 0..config.worker_threads {
            populations.push(next_gen.to_vec());
        }
    }
}

fn breed_from_pop<R:Rng>(breeding_group : &[Creature], r : &mut R, num_cities : u32) -> Creature {
    assert!(breeding_group.len() > 1);
    let id1 = r.gen::<usize>() % breeding_group.len();
    loop {
        let id2 = r.gen::<usize>() % breeding_group.len();
        if id1 != id2 {
            return breeding_group[id1].breed_with(&breeding_group[id2], r, num_cities);
        }
    }
}

fn fresh_group<R : Rng>(pop_size : usize, r : &mut R, num_cities : u32) -> Vec<Creature> {
    let mut population : Vec<Creature> = Vec::new();
    for _ in 0..pop_size {
        population.push(Creature::new_random(r, num_cities));
    }
    population
}

fn evolve<R:Rng>(start : &Vec<Creature>, mut r : R, distances : &DistMap, num_cities : u32, config : &setup::Config) -> Vec<Creature> {
    let mut population = Vec::new();
    for z in start{
        population.push(z.clone());
    }
    let pop_size = population.len();
    let breed_group_size = pop_size/2;
    let mut offspring = Vec::new();

    // generations
    for _ in 0..config.generations {

        sort_pop_objectively(&mut population, distances);
        while population.len() > breed_group_size {
            population.pop();
        }
        while population.len() + offspring.len() < pop_size {
            offspring.push(
                breed_from_pop(& population[..breed_group_size], &mut r, num_cities)
            );
        }
        while let Some(mut o) = offspring.pop(){
            if !r.gen_weighted_bool(3) {
                o.mutate(&mut r, num_cities);
            }
            population.push(o);
        }
    }
    population
}
use std::cmp::Ordering;

fn sort_pop_objectively(population : &mut Vec<Creature>, distances : &DistMap){
    population.sort_by(
        |c1 : &Creature, c2 : &Creature|{
            if let Some(ord) = c1.obj_func(&distances)
                .partial_cmp(&c2.obj_func(&distances)){
                    if ord == Ordering::Equal {c1.cities.cmp(& c2.cities)} else {ord}
                }
            else{
                c1.cities.cmp(& c2.cities)
            }
        }
    )
}
