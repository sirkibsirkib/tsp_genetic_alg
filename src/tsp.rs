use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::io::BufRead;
use std::fs::File;
use std::iter::FromIterator;
use std::str::FromStr;
use std;
use std::io::{Error, ErrorKind};
use bidir_map::BidirMap;



#[derive(PartialEq)]
pub struct Point {
    x : f32,
    y : f32,
}
pub type CityMap = BidirMap<u32, String>;
pub type PointMap = HashMap<u32, Point>;
pub type DistMap = HashMap<(u32, u32), f32>;

pub fn read_point_map(path : &str) -> Result<(CityMap, PointMap), std::io::Error> {
    /*
    format:
    doc = [line]*
    line =  [ws] <name1:String> [ws] '|'  [ws] <x:f32>  [ws] '|' [ws] <y:f32>  [ws] '\n'
    ws = (' ' | '\t')*
    */
    let mut city_map : CityMap = BidirMap::new();
    let mut point_map : PointMap = HashMap::new();
    let mut next_id : u32 = 0;

    let f = File::open(path)?;
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let res = Vec::from_iter(l.split("|").map(|x| String::from(x).trim().to_owned()));
        if res.len() != 3 {
            return Err(Error::new(std::io::ErrorKind::InvalidData, "Line has wrong number of fields"));
        }
        let c_id : u32 = if city_map.contains_second_key(&res[0]){
            * city_map.get_by_second(&res[0]).unwrap()
        } else {
            city_map.insert(next_id, res[0].to_owned());
            next_id += 1;
            next_id - 1
        };

        point_map.insert(
            c_id,
            Point{
                x : f32::from_str(&res[1]).expect("Couldn't parse float!"),
                y : f32::from_str(&res[2]).expect("Couldn't parse float!"),
            },
        );

    }
    Ok((city_map, point_map))
}

macro_rules! sqr {
    ($x:expr) => {{
        $x * $x
    }}
}
pub fn point_map_to_dist_map(city_map : &CityMap, point_map : &PointMap) -> DistMap {
    let mut dist_map : DistMap = HashMap::new();
    for cid_1 in city_map.first_col(){
        for cid_2 in city_map.first_col(){
            let (p1, p2) = (point_map.get(cid_1).unwrap(), point_map.get(cid_2).unwrap());
            dist_map.insert(
                (*cid_1, *cid_2),
                (sqr!(p1.x - p2.x) + sqr!(p1.y - p2.y)).sqrt(),
            );
        }
    }
    dist_map
}

pub fn write_dist_map(city_map : &CityMap, dist_map : &DistMap, path : &str) -> Result<(), Error>{
    use std::fs::File;
    use std::io::Write;

    let mut f = File::create(path)?;
    let mut writer = BufWriter::new(&f);

    for cid_1 in city_map.first_col(){
        for cid_2 in city_map.first_col(){
            write!(
                &mut writer,
                "{}\t|{}\t|{}\n",
                city_map.get_by_first(cid_1).unwrap(),
                city_map.get_by_first(cid_2).unwrap(),
                dist_map.get(&(*cid_1, *cid_2)).unwrap(),
            );
        }
    }
    Ok(())
}

pub fn read_dist_map(path : &str) -> Result<(CityMap, DistMap), std::io::Error> {
    /*
    format:
    doc = [line]*
    line =  [ws] <name1:String> [ws] '|'  [ws] <name2:String>  [ws] '|' [ws] <dist:f32>  [ws] '\n'
    ws = (' ' | '\t')*
    */
    let mut city_map : CityMap = BidirMap::new();
    let mut dist_map : DistMap = HashMap::new();
    let mut next_id : u32 = 0;

    let f = File::open(path)?;
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let res = Vec::from_iter(l.split("|").map(|x| String::from(x).trim().to_owned()));
        if res.len() != 3 {
            return Err(Error::new(std::io::ErrorKind::InvalidData, "Line has wrong number of fields"));
        }

        let c1_id : u32 = if city_map.contains_second_key(&res[0]){
            * city_map.get_by_second(&res[0]).unwrap()
        } else {
            city_map.insert(next_id, res[0].to_owned());
            next_id += 1;
            next_id - 1
        };

        let c2_id : u32 = if city_map.contains_second_key(&res[1]){
            * city_map.get_by_second(&res[1]).unwrap()
        } else {
            city_map.insert(next_id, res[1].to_owned());
            next_id += 1;
            next_id - 1
        };

        dist_map.insert(
            (c1_id, c2_id),
            f32::from_str(&res[2]).expect("Couldn't parse float!"),
        );
    }
    Ok((city_map, dist_map))
}
