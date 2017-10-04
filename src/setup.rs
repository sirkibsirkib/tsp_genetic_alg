use num_cpus;
use std::cmp::max;
use std::io::{BufReader};
use std::io::BufRead;
use std::fs::File;
// use std::io::Error;
// use std::collections::HashMap;
use std::iter::FromIterator;
// use std::str::FromStr;
// use std;
// use bidir_map::BidirMap;


#[derive(Debug, PartialEq)]
pub enum InMode {
    DistMode, CoordMode,
}

#[derive(Debug)]
pub struct Config{
    pub eras : usize,
    pub generations : usize,
    pub population : usize,
    pub worker_threads : usize,
    pub in_path : String,
    pub dist_path : String,
    pub in_mode : InMode,
}

fn default_config() -> Config {
    let mut config = Config {
        eras : 100,
        generations : 100,
        population : 50,
        worker_threads : max(1, num_cpus::get()-1),
        in_path : "INVALID".to_owned(),
        dist_path : String::new(),
        in_mode : InMode::DistMode,
    };
    if let Ok(f) = File::open("./default_config.txt") {
        println!("./default_config.txt found!");
        let file = BufReader::new(&f);
        for line in file.lines() {
            let l = line.unwrap();
            let res = Vec::from_iter(l.split(":").map(|x| String::from(x).trim().to_owned()));
            if res.len() != 2 {
                break;
            }
            match res[0].as_ref() {
                "eras" => if let Ok(val) = res[1].parse::<usize>() {config.eras = val},
                "generations" => if let Ok(val) = res[1].parse::<usize>() {config.generations = val},
                "population" => if let Ok(val) = res[1].parse::<usize>() {config.population = val},
                "worker_threads" => if let Ok(val) = res[1].parse::<usize>() {config.worker_threads = val},
                "in_path" => config.in_path = res[1].to_owned(),
                "dist_path" => config.dist_path = res[1].to_owned(),
                "in_mode" => if res[1] == "CoordMode" {config.in_mode = InMode::CoordMode},
                _ => (),
            }
        }
    } else {
        println!("./default_config.txt NOT found!");
    }
    config
}

pub fn configure() -> Config {
    let mut config = default_config();
    let matches = clap_app!(TSP_Genetic_Alg =>
            (version: "1.0")
            (author: "NAME <email>")
            (about: "decript.")

            (@arg input_path: +takes_value "path of the input txt file")
            (@arg dist_path: +takes_value "writes the resulting distance map to this path (optional)")
            (@arg CoordMode: -c --coord "Enables coordinate mode. Input file will be parsed as coordinates rather than a->b distances")
            (@arg worker_threads: -w --workers +takes_value "Choose number of worker_threads. defaults to cores - 1")
            (@arg eras: -e --eras +takes_value "set the number of eras (globally-synched rounds of breeding)")
            (@arg generations: -g --generations +takes_value "set the number of generations (number of selection rounds in a thread per era)")
            (@arg population: -p --population +takes_value "set the population size per thread")
        ).get_matches();


    if let Some(s) = matches.value_of("input_path") {
        config.in_path = s.to_owned();
    } else {
        if config.in_path == "INVALID"{
            panic!("NO IN PATH GIVEN IN CONFIG FILE NOR INPUT");
        }
    }
    if let Some(s) = matches.value_of("dist_path") {
        config.dist_path = s.to_owned();
    }
    if let Some(s) = matches.value_of("worker_threads") {
        config.worker_threads = s.parse().unwrap();
    };

    if let Some(s) = matches.value_of("eras") {
        config.eras = s.parse().unwrap();
    };

    if let Some(s) = matches.value_of("generations") {
        config.generations = s.parse().unwrap();
    };

    if let Some(s) = matches.value_of("population") {
        config.population = s.parse().unwrap();
    };

    if matches.occurrences_of("CoordMode") >= 1 {
        config.in_mode = InMode::CoordMode;
    }
    config
}
