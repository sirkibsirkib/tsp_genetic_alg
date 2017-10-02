macro_rules! sqr {
    ($x:expr) => {{
        $x * $x
    }}
}

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

extern crate rand;

use std::collections::HashMap;
use rand::{SeedableRng, Rng, Isaac64Rng};

type DistMap = HashMap<(usize, usize), f32>;


fn distance_between(dmap : &DistMap, c1 : City, c2 : City) -> f32 {
    if let Some(distance) = dmap.get(&(c1.id, c2.id)) {
        *distance
    } else if let Some(distance) = dmap.get(&(c2.id, c1.id)) {
        *distance
    } else {
        0.0
    }
}

fn main() {
    let mut rng = Isaac64Rng::from_seed(&[5,4,3,3,5,6]);

    let names = vec!["Amsterdam", "Vlissingen", "Cologne",
        "Haarlem", "Leiden", "Johannesburg", "Vancouver", "Washington"];

    let mut distances : DistMap = HashMap::new();
    let mut cities : Vec<City> = Vec::new();
    for (i, n) in names.iter().enumerate() {
        let p = Point{x : rng.gen(), y : rng.gen()};
        cities.push(City {id : i, name : (*n).to_owned(), p : p});
    }
    for (i, el1) in cities.iter().enumerate() {
        for el2 in cities[i+1..].iter() {
            let key = (el1.id, el2.id);
            distances.insert(
                key,
                el1.p.distance_to(&el2.p) * (rng.gen::<f32>() * 0.4 + 0.8),
            );
            println!("({}, {}) --> {}", el1.name, el2.name, distances.get(&key).unwrap());
        }
    }
    println!("Hello, world!");
}
