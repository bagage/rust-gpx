extern crate rquery;
extern crate chrono;
#[macro_use(value_t)]
extern crate clap;
extern crate xmltree;
extern crate regex;

use rquery::Document;
use chrono::*;
use std::f64;

use xmltree::Element;
use std::fs::File;
use std::io::prelude::*;

use regex::Regex;

mod cli;

struct Point {
    lat: f64,
    lon: f64,
}

fn distance(a: &Point, b: &Point) -> f64 {
    let r = 6371.;
    let d_lat = (b.lat - a.lat).to_radians();
    let d_lon = (b.lon - a.lon).to_radians();
    let a = (d_lat / 2.).sin() * (d_lat / 2.).sin() +
            a.lat.to_radians().cos() * b.lat.to_radians().cos() * (d_lon / 2.).sin() *
            (d_lon / 2.).sin();
    let c = 2. * a.sqrt().atan2((1. - a).sqrt());
    let d = r * c * 1000.; // Distance in meters

    return d;
}

fn compute_best(points: &Vec<(Point, DateTime<UTC>)>,
                time_threshold: Option<Duration>,
                distance_threshold: f64)
                -> f64 {
    let time_mode = time_threshold != None;
    let mut best: f64 = if time_mode {
        0.0
    } else {
        f64::INFINITY
    };

    let mut i = points.iter();
    while let Some(v) = i.next() {
        let (ref p1, time) = *v;
        let mut current_time = 0.;
        let mut prev_time = time.timestamp();
        let mut current_distance = 0.;

        let mut prev_point = p1;
        for v2 in i.clone() {
            let (ref p2, time2) = *v2;

            let this_distance = distance(&prev_point, &p2);
            let this_time = (time2.timestamp() - prev_time) as f64;

            if time_mode {
                let threshold = time_threshold.unwrap().num_seconds() as f64;
                if current_time + this_time >= threshold {
                    // make linear interpolation
                    let alpha = (threshold - current_time) / this_time;
                    assert!(0. < alpha && alpha <= 1.);
                    current_distance += this_distance * alpha;

                    best = best.max(current_distance);
                    break;
                }
            } else {
                let threshold = distance_threshold;
                if current_distance + this_distance >= threshold {
                    let alpha = (threshold - current_distance) / this_distance;
                    assert!(0. < alpha && alpha <= 1.);
                    current_time += this_time * alpha;
                    best = best.min(current_time);
                    break;
                }
            }
            current_distance += this_distance;
            current_time += this_time;
            prev_time = time2.timestamp();
            prev_point = p2;
        }
    }
    if time_mode {
        println!("Best for {}s time was {}m", time_threshold.unwrap().num_seconds(), best);
    } else {
        println!("Best for {}m distance was {}s", distance_threshold, best);
    }
    return best;
}

fn analyze(gpx_file: &str, distance: str, time: str) {
    let document = Document::new_from_xml_file(gpx_file).unwrap();
    let points: Vec<(Point, DateTime<UTC>)> = document.select_all("trkpt")
        .unwrap()
        .map(|el| {
            let lat: f64 = el.attr("lat").unwrap().to_string().parse::<f64>().unwrap();
            let lon: f64 = el.attr("lon").unwrap().to_string().parse::<f64>().unwrap();
            let time = el.select("time").unwrap().text().parse::<DateTime<UTC>>().unwrap();
            (Point {
                 lat: lat,
                 lon: lon,
             },
             time)
        })
        .collect();


    if Some(distance) {
        let distance_threshold = 0.;
        // parse distance from human-readable syntax 1km or 100m to meters
        if distance.ends_with("km") {
            distance_threshold = 1000. * distance.truncate(distance.len() - 2).parse::<f64>();
        } else if distance.ends_with("m") {
            distance_threshold = distance.truncate(distance.len() - 1).parse::<f64>();
        } else {
            distance_threshold = distance.parse::<f64>();
        }
        compute_best(&points, None, distance_threshold);
    } else {
        let re = Regex::new(r"^\d{2}.\d{2}.\d{2}$").unwrap();
        let group = re.captures_iter(time);
        println!("{}", group[0]*3600+group[1]*60+group[2]);
        compute_best(&points, Some(Duration::seconds(group[0]*3600+group[1]*60+group[2])), 0);
    }
}

fn merge(files: &Vec<&str>, output: &str) {
    if files.len() < 2 {
        println!("Expected at least 2 files, got {}", files.len());
        return
    }

    // sort files by <time> metadata attribute
    let mut sorted_files : Vec<(DateTime<UTC>, &str)> = Vec::new();
    for file in files.iter() {
        let document = Document::new_from_xml_file(file).unwrap();
        let time = document.select("metadata").unwrap().select("time").unwrap().text().parse::<DateTime<UTC>>().unwrap();

        let new_elem = (time, *file);
        let pos = sorted_files.binary_search(&new_elem).unwrap_or_else(|e| e);
        sorted_files.insert(pos, new_elem);
    }

    let mut f = File::open(sorted_files[0].1).unwrap();
    let mut buffer_root = String::new();
    f.read_to_string(&mut buffer_root);
    let mut gpx_root = Element::parse(buffer_root.as_bytes()).unwrap();

    for tuple in sorted_files.iter().skip(1) {
        let mut trk = gpx_root.get_mut_child("trk").expect("Cannot find 'trk' XML element");
        let mut f2 = File::open(tuple.1).unwrap();
        let mut buffer_root2 = String::new();
        f2.read_to_string(&mut buffer_root2);

        let gpx_root2 = Element::parse(buffer_root2.as_bytes()).unwrap();
        {
            let trk2 = gpx_root2.get_child("trk").expect("Cannot find 'trk' XML element");
            for trkpt2 in trk2.children.clone() {
                trk.children.push(trkpt2);
            }
        }

    }

    let output_file = File::create(output).unwrap();
    gpx_root.write(&output_file);
    output_file.sync_all().unwrap();
}

fn main() {
    let matches = cli::build_cli().get_matches();

    match matches.subcommand() {
         ("analyze", Some(analyze_matches)) => analyze(analyze_matches.value_of("gpx-file").unwrap(),
                analyze_matches.value_of("distance"),
                analyze_matches.value_of("time")),
        ("merge", Some(merge_matches)) => merge(&merge_matches.values_of("gpx-files").unwrap().collect(), &merge_matches.value_of("output-file").unwrap()),
        ("", None) => println!("No command requested"),
        _ => unreachable!(),
    }
}
