macro_rules! sqr {
    ($x:expr) => {{
        $x * $x
    }}
}

const CITY_NAMES : &'static [&'static str] =
    &["Amsterdam", "Vlissingen", "Cologne", "Copenhagen", "Duivendrecht", "Gulpen", "Wageningen",
    "Bethlehem", "Munich", "Loewen", "Maastricht", "Bloemfontein", "Stellenbosch", "Amstelveen",
    "Maasdoorn", "Zwolle", "Leeuwaarden", "Arnhem", "Boisheim", "Kaldenkirchen", "Celle",
    "Nijmegen", "Windhoek", "Monaco", "London", "Berlin", "Viersen", "Krefeld", "Venlo", "Hamburg",
    "Duesseldorf", "Shanghai", "Ottawa", "Dublin", "Dubai", "Houston", "Austin", "Venice",
    "Vienna", "Bethlehem", "Potsdam"];
const NUM_CITIES : usize = 41;
const NUM_THREADS : usize = 4;
const POP_SIZE : usize = 20;
const GENERATIONS : usize = 50;
const ERAS : usize = 20;

struct Point {
    x : f32,
    y : f32,
}
impl Point {
    fn distance_to(&self, other : &Point) -> f32 {
        return (sqr!(self.x - other.x) + sqr!(self.y - other.y)).sqrt();
    }
}

struct City{
    id : usize,
    name : String,
    p : Point,
}

impl PartialEq for City {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
use std::hash::{Hash, Hasher};
impl Eq for City {}
impl Hash for City {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
extern crate cue;
extern crate rand;

use std::collections::HashMap;
use rand::{SeedableRng, Rng, Isaac64Rng};

type DistMap = HashMap<(usize, usize), f32>;


fn distance_between(dmap : &DistMap, cid1 : usize, cid2 : usize) -> f32 {
    if let Some(distance) = dmap.get(&(cid1, cid2)) {
        *distance
    } else if let Some(distance) = dmap.get(&(cid2, cid1)) {
        *distance
    } else {
        0.0
    }
}

fn make_cities(names : &[&str]) -> Vec<City> {
    let mut rng = Isaac64Rng::from_seed(&[32,56,2]);
    let mut cities : Vec<City> = Vec::new();
    for (i, n) in names.iter().enumerate() {
        let p = Point{x : rng.gen::<f32>()*100.0, y : rng.gen::<f32>()*100.0};
        cities.push(City {id : i, name : (*n).to_owned(), p : p});
    }
    cities
}

fn make_dmap(cities : &Vec<City>) -> DistMap {
    let mut rng = Isaac64Rng::from_seed(&[5,4,3,3,5,6]);
    let mut distances : DistMap = HashMap::new();
    for (i, el1) in cities.iter().enumerate() {
        for el2 in cities[i+1..].iter() {
            let key = (el1.id, el2.id);
            distances.insert(
                key,
                el1.p.distance_to(&el2.p) * (rng.gen::<f32>() * 0.4 + 0.8),
            );
        }
    }
    distances
}

#[derive(Debug)]
struct Creature {
    city_sequence : Vec<usize>,
}

impl Creature {
    fn print_route (&self) {
        for &cid in self.city_sequence.iter() {
            print!("{:?} --> ", CITY_NAMES[cid]);
        }
    }

    fn mutate<R : Rng>(&mut self, r : &mut R) {
        let ind_a = r.gen::<usize>() % NUM_CITIES;
        let ind_b = r.gen::<usize>() % NUM_CITIES;
        self.city_sequence.swap(ind_a, ind_b);
    }

    fn breed_with<R : Rng>(&self, other : &Creature, r : &mut R) -> Creature {
        let offset = r.gen::<usize>() % NUM_CITIES;
        let mut v = Vec::new();
        let copylen = self.city_sequence.len() / 2;
        for i in 0..copylen {
            let index = (i + offset) % NUM_CITIES;
            v.push(self.city_sequence[index]);
        };
        for x in other.city_sequence.iter() {
            if ! v.contains(x) {
                v.push(*x);
            }
        }
        Creature {city_sequence : v}
    }

    fn obj_func(&self, dmap : &DistMap) -> f32 {
        let mut tot = 0.0;
        let mut prev = *self.city_sequence.last().unwrap();
        for &city in self.city_sequence[0..].iter() {
            let cost = distance_between(dmap, prev, city);
            tot += cost;
            prev = city;
        }
        tot
    }
}

fn random_creature<R : Rng>(r : &mut R) -> Creature {
    let mut v : Vec<usize> = Vec::new();
    for x in 0..NUM_CITIES {
        v.push(x);
    }
    r.shuffle(&mut v);
    Creature {city_sequence : v}
}

fn child_rng<R : Rng>(proto_rng : &mut R) -> Isaac64Rng {
    let mut seed : [u64; 10] = [0; 10];
    for i in 0..10 {
        seed[i] = proto_rng.gen();
    };
    Isaac64Rng::from_seed(&seed)
}

fn main() {
    use std::io::Write;
    use std::io::stdout;
    assert_eq!(CITY_NAMES.len(), NUM_CITIES);
    let mut rng = Isaac64Rng::from_seed(&[1,2,3,4,5,6,8]);

    let cities = make_cities(&CITY_NAMES);
    let distances = make_dmap(&cities);
    let mut population = fresh_group(20, &mut rng);

    let mut populations : Vec<Vec<Creature>> = Vec::new();
    for _ in 0..NUM_THREADS {
        populations.push(fresh_group(POP_SIZE, &mut rng));
    }


                //SEQUENTIAL
                let mut pop = fresh_group(POP_SIZE, &mut rng);
                for _ in 0..ERAS {
                    pop = evolve(pop, &mut rng, &distances);
                }
                for b in pop.iter().take(1) {
                    println!("{:?} :: {}", b, b.obj_func(&distances));
                }
                pop[0].print_route();


    //PARALLEL

    // for era in 0..30 {
    //     use std::thread;
    //
    //     let mut next_populations : Vec<Vec<Creature>>  = Vec::new();
    //
    //     cue::pipeline(
    //         "idk",
    //         NUM_THREADS,
    //         populations.into_iter(),
    //         |input| {evolve(input, rng, &distances)},
    //         |output| {next_populations.push(output)},
    //     );
    //
    //     let next_gen : Vec<Creature>;
    //     for p in populations{
    //         for z in p {
    //             next_gen.push(z);
    //         }
    //     }
    //     sort_pop_objectively(&mut next_gen, &distances);
    // }
}

fn breed_from_pop<R:Rng>(breeding_group : &[Creature], r : &mut R) -> Creature {
    assert!(breeding_group.len() > 1);
    let id1 = r.gen::<usize>() % breeding_group.len();
    loop {
        let id2 = r.gen::<usize>() % breeding_group.len();
        if id1 != id2 {
            return breeding_group[id1].breed_with(&breeding_group[id2], r);
        }
    }
}

fn fresh_group<R : Rng>(pop_size : usize, r : &mut R) -> Vec<Creature> {
    let mut population : Vec<Creature> = Vec::new();
    for _ in 0..pop_size {
        population.push(random_creature(r));
    }
    population
}

fn evolve<R:Rng>(mut population : Vec<Creature>, r : &mut R, distances : &DistMap) -> Vec<Creature> {
    let pop_size = population.len();
    let breed_group_size = pop_size/2;
    let mut offspring = Vec::new();

    // generations
    for _ in 0..GENERATIONS {
        // print!("\n\nGEN {}\n", gen_id);
        sort_pop_objectively(&mut population, distances);
        // for p in population.iter().take(1) {
        //     println!("{:?} == {}", p, p.obj_func(&distances));
        // }
        while population.len() > breed_group_size {
            population.pop();
        }
        while population.len() + offspring.len() < pop_size {
            offspring.push(
                breed_from_pop(& population[..breed_group_size], r)
            );
        }
        while let Some(mut o) = offspring.pop(){
            o.mutate(r);
            population.push(o);
        }
        // population.push(random_creature(r));
    }
    population
}

fn sort_pop_objectively(population : &mut Vec<Creature>, distances : &DistMap){
    population.sort_by(
        |c1 : &Creature, c2 : &Creature|
        c1.obj_func(&distances)
            .partial_cmp(&c2.obj_func(&distances))
            .unwrap_or(std::cmp::Ordering::Equal)
    );
}
