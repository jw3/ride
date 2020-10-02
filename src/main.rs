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

use log::{debug, info, warn};

use actix::prelude::*;
use actix::clock::{delay_for, Duration};

use reqwest::Client;

struct Driver {
    uri: Option<String>,
    steptime: actix::clock::Duration,
    current_step: u64,
    total_steps: u64,
    traveled: f64, // distance traveled in meters
    previous_point: Option<Point<f64>>
}

impl Actor for Driver {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("I am alive!");
    }
}

impl Handler<WayPoint> for Driver {
    type Result = ();

    fn handle(&mut self, p: WayPoint, _ctx: &mut Context<Self>) {
        let pct = (self.current_step as f64 / self.total_steps as f64) * 100.0;
        let d = self.previous_point.map(|prev| prev.geodesic_distance(&p.pos)).unwrap_or(0.0);
        self.traveled += d;

        info!("{}: {:.1}m ({:.1}%)", self.current_step, self.traveled, pct);
        self.current_step += 1;
        self.previous_point = Some(p.pos);

        let e = Event {
            id: String::from("foo"),
            lon: format!("{:.6}", p.pos.x_y().0),
            lat: format!("{:.6}", p.pos.x_y().1)
        };
        let json = serde_json::to_string_pretty(&e).unwrap();

        if let Some(uri) = &self.uri {
            info!("{}", json);
            let r = reqwest::Client::new().post(uri).json(&e).send();
            // if let Err(e) = r {
            //     warn!("{}", e);
            // }
        }
        else {
            println!("{}", json);
        }

        delay_for(self.steptime);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct WayPoint {
    pos: Point<f64>
}

#[derive(Serialize)]
struct Event {
    id: String,
    lon: String,
    lat: String,
}

fn as_point(c: (f64, f64, f64)) -> Point<f64> {
    return Point::from([c.0, c.1]);
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

    let system = actix::System::new("ride");

    let mut dataset = Dataset::open(Path::new(&opts.gpkg)).unwrap();
    let layer = dataset.layer(0).unwrap();

    let kph = opts.speed;
    let mps = kph / 3.6;          // meters per second
    let int = opts.interval;      // interval of updates (from sensor)

    for feature in layer.features() {
        let pv: Vec<Point<f64>> = feature.geometry().get_point_vec().into_iter().map(as_point).collect();
        let l: LineString<f64> = LineString::from_iter(pv.iter().map(|p|p.0));

        let d0 = l.geodesic_length();
        debug!("distance: {}m", d0.round());

        let tts = d0 / mps;           // travel time in seconds
        let stp = tts / int as f64;   // steps total
        let ppu = 100.0 / stp;        // percent per update

        // use the factor value to increase the playback speed
        let step_length = actix::clock::Duration::from_millis(int * 1000 / opts.factor);

        let d = Driver {
            uri: opts.uri.clone(),
            steptime: step_length,
            current_step: 0,
            total_steps: stp as u64,
            traveled: 0.0,
            previous_point: None
        };
        let a = d.start();

        let p0 = pv.first().unwrap();
        a.do_send(WayPoint{ pos: *p0 });
        for s in 1..(stp as i64) {
            let p = s as f64 * ppu;
            let sp: Point<f64> = l.line_interpolate_point(&(p / 100.0)).x_y().into();
            a.do_send(WayPoint{ pos: sp });
        }
        let p1 = pv.last().unwrap();
        a.do_send(WayPoint{ pos: *p1 });
    }

    system.run();
}
