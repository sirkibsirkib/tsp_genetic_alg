
use std::fmt;


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
const POP_SIZE : usize = 30;
const GENERATIONS : usize = 22;
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

#[derive(Copy)]
struct Creature {
    city_sequence : [usize; NUM_CITIES],
}

impl fmt::Debug for Creature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Creature [").is_ok();
        for i in 0..NUM_CITIES {
            write!(f, "{}, ", self.city_sequence[i]).is_ok();
        }
        write!(f, "]")
    }
}

impl Clone for Creature {
    fn clone(&self) -> Self{
        let mut x = [0; NUM_CITIES];
        for i in 0..NUM_CITIES{
            x[i] = self.city_sequence[i];
        }
        Creature{city_sequence : x}
    }
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
        let mut v = [0 ; NUM_CITIES];
        let copylen = self.city_sequence.len() / 2;
        for i in 0..copylen {
            let index = (i + offset) % NUM_CITIES;
            v[i] = self.city_sequence[index];
        };
        let mut v_len = copylen;
        for x in other.city_sequence.iter() {
            if ! v.contains(x) {
                v[v_len] = *x;
                v_len += 1;
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
    let mut v = [0 ; NUM_CITIES];
    for x in 0..NUM_CITIES {
        v[x] = x;
    }
    r.shuffle(&mut v);
    Creature {city_sequence : v}
}

fn main() {
    assert_eq!(CITY_NAMES.len(), NUM_CITIES);
    let mut rng = Isaac64Rng::from_seed(&[1,2,3,4,5,6,8]);

    let cities = make_cities(&CITY_NAMES);
    let distances = make_dmap(&cities);

    let mut populations : Vec<Vec<Creature>> = Vec::new();
    for _ in 0..NUM_THREADS {
        populations.push(fresh_group(POP_SIZE, &mut rng));
    }

    for era in 0..ERAS {
        let mut next_populations : Vec<Vec<Creature>>  = Vec::new();
        cue::pipeline(
            "idk",
            NUM_THREADS,
            populations.iter(),
            |input| {
                evolve(input, rng, &distances)
            },
            |output| {
                next_populations.push(output)
            },
        );

        let mut next_gen : Vec<Creature> = Vec::new();
        for p in next_populations {
            for z in p {
                next_gen.push(z);
            }
        }
        sort_pop_objectively(&mut next_gen, &distances);
        next_gen.dedup_by(
            |a, b|
            order_city_seqs(&a.city_sequence, &b.city_sequence) == Ordering::Equal
        );
        while next_gen.len() > POP_SIZE {
            next_gen.pop();
        }
        while next_gen.len() < POP_SIZE {
            next_gen.push(random_creature(&mut rng));
        }

        println!("Best of Era {} : {:?} with {}", era, &next_gen[0], &next_gen[0].obj_func(&distances));
        if era == ERAS-1 {
            println!("\n\n");
            next_gen[0].print_route();
            break;
        }
        populations = Vec::new();
        for _ in 0..NUM_THREADS {
            populations.push(next_gen.to_vec());
        }
    }
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

fn evolve<R:Rng>(start : &Vec<Creature>, mut r : R, distances : &DistMap) -> Vec<Creature> {
    let mut population = Vec::new();
    for z in start{
        population.push(z.clone());
    }
    let pop_size = population.len();
    let breed_group_size = pop_size/2;
    let mut offspring = Vec::new();

    // generations
    for _ in 0..GENERATIONS {
        sort_pop_objectively(&mut population, distances);
        while population.len() > breed_group_size {
            population.pop();
        }
        while population.len() + offspring.len() < pop_size {
            offspring.push(
                breed_from_pop(& population[..breed_group_size], &mut r)
            );
        }
        while let Some(mut o) = offspring.pop(){
            o.mutate(&mut r);
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
                    if ord == Ordering::Equal {c1.city_sequence.cmp(& c2.city_sequence)} else {ord}
                }
            else{
                order_city_seqs(&c1.city_sequence, &c2.city_sequence)
            }
        }
    )
}

fn order_city_seqs(a : &[usize ; NUM_CITIES], b : &[usize ; NUM_CITIES]) -> Ordering {
    for i in 0..NUM_CITIES {
        match a[i].cmp(&b[i]) {
            Ordering::Equal => (),
            x => return x,
        }
    };
    Ordering::Equal
}
