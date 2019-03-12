extern crate rquery;
extern crate chrono;
#[macro_use(value_t)]
extern crate clap;
extern crate xmltree;
extern crate gpx;

use rquery::Document;

use chrono::prelude::Utc;
use chrono::DateTime;
use chrono::Duration;

use std::io::BufReader;
use std::f64;

use std::collections::HashMap;

use gpx::read;
use gpx::Gpx;

use xmltree::Element;
use std::fs::File;
use std::io::prelude::*;

mod cli;

struct Point {
    lat: f64,
    lon: f64,
    ele: f64,
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

fn format_duration(d: i64) -> String {
    let hour = d as i64 / 3600;
    let min = (d as i64 % 3600) / 60;
    let sec = d as i64 % 60;
    return format!("{}{}{}s", if hour > 0 { format!("{}h", hour) } else { "".to_string() },
                                          if min > 0 { format!("{}m", min) } else { "".to_string() },
                                          sec);
}

fn compute_elevation(points: &Vec<(Point, DateTime<Utc>)>,
                     start_time: DateTime<Utc>,
                     end_time: DateTime<Utc>)
                     -> [f64; 2] {
                                // d+ first, then d-
    let mut results : [f64; 2] = [0.; 2];

    let mut i = points.iter();
    let mut prev_ele = 0.0;
    while let Some(v) = i.next() {
        let (ref p1, time) = *v;
        if time < start_time {
            continue;
        } else if time > end_time {
            break;
        }

        if prev_ele != 0. {
            let delta = p1.ele - prev_ele;
            if delta > 0. { results[0] += delta; } else { results[1] += -delta; }
        }
        prev_ele = p1.ele;
    }
    return results;
}

fn compute_best(points: &Vec<(Point, DateTime<Utc>)>,
                time_threshold: Option<Duration>,
                distance_threshold: f64)
                -> (f64, [DateTime<Utc>; 2]) {
    let time_mode = time_threshold != None;
    //FIXME: proper Null initialization?
    let mut best_interval : [DateTime<Utc>; 2] = ["2017-06-27T18:16:08Z".parse::<DateTime<Utc>>().unwrap(); 2];
    let mut best: f64 = if time_mode { 0.0 } else { f64::INFINITY };

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
                    if current_distance > best {
                        best = current_distance;
                        best_interval = [time, time2];
                    }
                    break;
                }
            } else {
                let threshold = distance_threshold;
                if current_distance + this_distance >= threshold {
                    let alpha = (threshold - current_distance) / this_distance;
                    assert!(0. < alpha && alpha <= 1.);
                    current_time += this_time * alpha;
                    if current_time < best {
                        best = current_time;
                        best_interval = [time, time2];
                    }
                    break;
                }
            }
            current_distance += this_distance;
            current_time += this_time;
            prev_time = time2.timestamp();
            prev_point = p2;
        }
    }
    return (best, best_interval);
}

fn info(gpx_file: &str) {
    let file = File::open(gpx_file).unwrap();
    let gpx: Gpx = read(BufReader::new(file)).unwrap();
    
    let gpxStats: HashMap<&str, f64> = HashMap::new();

    println!("File: {file}", file=gpx_file);
    for track in gpx.tracks {
        for segment in track.segments {
            let mut segmentStats: HashMap<&str, f64> = HashMap::new();

            // let mut prev = &segment.points[0];
            for point in segment.points {
                // segmentStats.entry("Length 2D").or_insert(0) += distance(
                //     Point {
                //         lat: prev.point().lat(),
                //         lon: prev.point().lng(),
                //         ele: prev.elevation.unwrap_or(0.),
                //     },
                //     Point{
                //         lat: point.point().lat(),
                //         lon: point.point().lng(),
                //         ele: point.elevation.unwrap_or(0.),
                //     });
                // segmentStats.entry("Moving time").or_insert(0) += point.time.timestamp() - prev.time.timestamp();
                // segmentStats.entry("Stopped time").or_insert(0) += 0;
                let maxSpeed = segmentStats.entry("Max speed").or_insert(0.);
                *maxSpeed = maxSpeed.max(point.speed.unwrap_or(0.));
                // segmentStats.entry("Total uphill").or_insert(0) += if point.elevation > 0 { point.elevation } else { 0 };
                // segmentStats.entry("Total downhill").or_insert(0) += if point.elevation < 0 { -point.elevation } else { 0 };
                // prev = point;
            }
            // segmentStats.insert("Started", segment.points.first().unwrap().time.unwrap().to_string());
            // segmentStats.insert("Ended", segment.points.last().unwrap().time.unwrap().to_string());
            // segmentStats.insert("Points", segment.points.len());
            // segmentStats.insert("Avg distance between points", segmentStats.entry("Length 2D").or_insert(0.) / segmentStats.entry("Points"));

            for (key, value) in segmentStats {
                println!("\t\t{}: {}", key, value);
            }

        }
    }
    for (key, value) in gpxStats {
        println!("\t{}: {}", key, value);
    }
}

fn analyze(gpx_file: &str, distance: f64, time: i64) {
    let document = Document::new_from_xml_file(gpx_file).unwrap();
    let points: Vec<(Point, DateTime<Utc>)> = document.select_all("trkpt")
        .unwrap()
        .map(|el| {
            let lat: f64 = el.attr("lat").unwrap().to_string().parse::<f64>().unwrap();
            let lon: f64 = el.attr("lon").unwrap().to_string().parse::<f64>().unwrap();
            let ele: f64 = el.select("ele").unwrap().text().parse::<f64>().unwrap();
            let time = el.select("time").unwrap().text().parse::<DateTime<Utc>>().unwrap();
            (Point {
                 lat: lat,
                 lon: lon,
                 ele: ele,
             },
             time)
        })
        .collect();

    let distance_threshold = distance;
    let time_threshold = time;

    let (best, best_interval) = compute_best(&points, if time_threshold > 1 { Some(Duration::seconds(time_threshold)) } else { None }, distance_threshold);
    let ele = compute_elevation(&points, best_interval[0], best_interval[1]);
    if time > 1 {
        println!("Best for {} time was {}m ({:.0}d+ / {:.0}d-) in interval {} - {}", format_duration(time_threshold), best, ele[0], ele[1], best_interval[0], best_interval[1]);
    } else {
        println!("Best for {}m distance was {} ({:.0}d+ / {:.0}d-) in interval {} - {}", distance_threshold, format_duration(best as i64), ele[0], ele[1], best_interval[0], best_interval[1]);
    }
}

fn merge(files: &Vec<&str>, output: &str) {
    if files.len() < 2 {
        println!("Expected at least 2 files, got {}", files.len());
        return
    }

    // sort files by <time> metadata attribute
    let mut sorted_files : Vec<(DateTime<Utc>, &str)> = Vec::new();
    for path in files.iter() {
        let file = File::open(path).unwrap();
        let gpx: Gpx = read(BufReader::new(file)).unwrap();

        let time: DateTime<Utc> = match gpx.metadata { 
            Some(m) => m.time,
            None => gpx.tracks[0].segments[0].points[0].time,
        }.unwrap();

        let new_elem = (time, *path);
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
                value_t!(analyze_matches, "distance", f64).unwrap_or(0.),
                value_t!(analyze_matches, "time", i64).unwrap_or(0)),
        ("merge", Some(merge_matches)) => merge(&merge_matches.values_of("gpx-files").unwrap().collect(), &merge_matches.value_of("output-file").unwrap()),
        ("info", Some(info_matches)) => info(info_matches.value_of("gpx-file").unwrap()),
        ("", None) => println!("No command requested"),
        _ => unreachable!(),
    }
}
