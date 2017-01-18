
use clap::{App, Arg};

pub fn build_cli() -> App<'static, 'static> {
    App::new("gpxanalyzer")
        .version("0.1.0")
        .about("Compute best distance and/or best time legs")
        .arg(Arg::with_name("gpx-file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("GPX file to analyze")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("distance")
            .short("d")
            .long("distance")
            .value_name("DISTANCE_THRESHOLD")
            .help("Find best time for this distance in meters")
            .takes_value(true))
        .arg(Arg::with_name("time")
            .short("t")
            .long("time")
            .value_name("TIME_THRESHOLD")
            .help("Find best distance for this time in seconds")
            .takes_value(true))
}
