extern crate rquery;
extern crate chrono;
#[macro_use(value_t)]
extern crate clap;

use rquery::Document;
use chrono::*;
use std::f64;

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

fn analyze(gpx_file: &str, distance: f64, time: i64) {
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

    let distance_threshold = distance;
    let time_threshold = time;
    compute_best(&points, if time_threshold > 1 { Some(Duration::seconds(time_threshold)) } else { None }, distance_threshold);
}

fn merge(files: &str) { // &Vec<&str>) {
}

fn main() {
    let matches = cli::build_cli().get_matches();

    match matches.subcommand() {
         ("analyze", Some(analyze_matches)) => analyze(analyze_matches.value_of("gpx-file").unwrap(),
                value_t!(analyze_matches, "distance", f64).unwrap_or(0.),
                value_t!(analyze_matches, "time", i64).unwrap_or(0)),
        ("merge", Some(merge_matches)) => merge(merge_matches.value_of("gpx-files").unwrap()),
        ("", None) => println!("No command requested"),
        _ => unreachable!(),
    }
}
