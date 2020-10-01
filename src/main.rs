use std::iter::FromIterator;
use std::path::Path;
use std::thread;
use std::time;

use clap::Clap;
use gdal::vector::Dataset;
use geo::{LineString, Point};
use geo::algorithm::line_interpolate_point::LineInterpolatePoint;
use geo::prelude::*;
use serde::Serialize;

use log::{info, warn};

#[derive(Serialize)]
struct Event {
    id: String,
    lon: String,
    lat: String,
}

fn as_point(c: (f64, f64, f64)) -> Point<f64> {
    return Point::from([c.0, c.1]);
}

fn event(sp: &Point<f64>, opt_uri: &Option<String>) {
    let e = Event {
        id: String::from("foo"),
        lon: sp.x_y().0.to_string(),
        lat: sp.x_y().1.to_string()
    };

    let json = serde_json::to_string_pretty(&e).unwrap();
    info!("{}", json);

    if let Some(uri) = opt_uri {
        let r = reqwest::blocking::Client::new().post(uri).json(&e).send();
        if let Err(e) = r {
            warn!("{}", e);
        }
    }
}

/// Simulated sensor riding along geo features.
#[derive(Clap)]
#[clap(version = "v0.1.0")]
struct Opts {
    /// uri to POST events to
    #[clap(short, long)]
    uri: Option<String>,

    /// simulation playback speed factor
    #[clap(short, long, default_value = "1")]
    factor: u64,

    /// sensor travel time in kilometers per hour
    #[clap(short, long, default_value = "10.0")]
    speed: f64,

    /// simulated seconds between sensor updates
    #[clap(short, long, default_value = "1")]
    interval: u64,

    /// GeoPackage containing vector data
    gpkg: String,
}

fn main() {
    let opts: Opts = Opts::parse();
    env_logger::init();

    let mut dataset = Dataset::open(Path::new(&opts.gpkg)).unwrap();
    let layer = dataset.layer(0).unwrap();

    for feature in layer.features() {
        let pv: Vec<Point<f64>> = feature.geometry().get_point_vec().into_iter().map(as_point).collect();
        let l: LineString<f64> = LineString::from_iter(pv.iter().map(|p|p.0));
        let d0 = l.geodesic_length();
        info!("distance: {}m", d0.round());

        let kph = opts.speed;
        let mps = kph / 3.6;          // meters per second
        let tts = d0 / mps;           // travel time in seconds
        let int = opts.interval;      // interval of updates (from sensor)
        let stp = tts / int as f64;   // steps total
        let ppu = 100.0 / stp;        // percent per update

        // use the factor value to increase the playback speed
        let step_length = time::Duration::from_millis(int * 1000 / opts.factor);

        info!("{}: {}m ({}%)", 0, 0.0, 0);
        let p0 = pv.first().unwrap();
        event(&p0, &opts.uri);
        thread::sleep(step_length);

        let mut traveled = 0.0;
        let mut previous = Point::new(p0.x_y().0, p0.x_y().1);
        for s in 1..(stp as i64) {
            let p = s as f64 * ppu;
            let sp: Point<f64> = l.line_interpolate_point(&(p / 100.0)).x_y().into();
            traveled += previous.geodesic_distance(&sp);
            previous = sp;
            info!("{}: {:.1}m ({:.0}%)", s, traveled, p);
            event(&sp, &opts.uri);
            thread::sleep(step_length);
        }
        info!("{}: {:.1}m ({}%)", 5, d0, 100);
        let p1 = pv.last().unwrap();
        event(&p1, &opts.uri);
        thread::sleep(step_length);
    }
}
