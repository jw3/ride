use std::iter::FromIterator;
use std::path::Path;

use actix::fut::wrap_future;
use actix::prelude::*;
use clap::Clap;
use futures::stream;
use gdal::vector::Dataset;
use geo::{LineString, Point};
use geo::algorithm::line_interpolate_point::LineInterpolatePoint;
use geo::prelude::*;
use log::{debug, info, warn};
use serde::Serialize;
use tokio::time::throttle;


/// Simulated sensor riding along geo features.
#[derive(Clap)]
#[clap(version = "v0.1.0")]
struct Opts {
    /// GeoPackage containing vector data
    gpkg: String,

    /// name of layer to select features from
    #[clap(short, long)]
    layer: Option<String>,

    /// simulation playback speed factor
    #[clap(short, long, default_value = "1")]
    factor: u64,

    /// sensor travel time in kilometers per hour
    #[clap(short, long, default_value = "10.0")]
    speed: f64,

    /// simulated seconds between sensor updates
    #[clap(short, long, default_value = "2")]
    interval: u64,

    /// uri to POST events to
    #[clap(short, long)]
    uri: Option<String>
}

#[derive(Message)]
#[rtype(result = "()")]
struct WayPoint {
    id: String,
    pos: Point<f64>
}

#[derive(Serialize)]
struct Event {
    id: String,
    lon: String,
    lat: String,
}

struct Driver {
    uri: Option<String>,
    current_step: u64,
    total_steps: u64,
    traveled: f64, // distance traveled in meters
    previous_point: Option<Point<f64>>
}

impl Actor for Driver {
    type Context = Context<Self>;
}

impl StreamHandler<WayPoint> for Driver {
    fn handle(&mut self, p: WayPoint, ctx: &mut Context<Self>) {
        let pct = (self.current_step as f64 / self.total_steps as f64) * 100.0;
        let d = self.previous_point.map(|prev| prev.geodesic_distance(&p.pos)).unwrap_or(0.0);
        self.traveled += d;

        info!("{}: {:.1}m ({:.1}%)", self.current_step, self.traveled, pct);
        self.current_step += 1;
        self.previous_point = Some(p.pos);

        let e = Event {
            id: p.id,
            lon: format!("{:.6}", p.pos.x_y().0),
            lat: format!("{:.6}", p.pos.x_y().1)
        };
        let json = serde_json::to_string_pretty(&e).unwrap();

        if let Some(uri) = &self.uri {
            info!("{}", json);
            let f = reqwest::Client::new().post(uri).json(&e).send();
            let af = wrap_future(f).map(move |res, _actor: &mut Self, _ctx: &mut Context<Self>| {
                match res {
                    Ok(_) => (),
                    Err(err) => warn!("{}", err)
                }
            });
            ctx.spawn(af);
        }
        else {
            println!("{}", json);
        }
    }
}

fn as_point(c: (f64, f64, f64)) -> Point<f64> {
    return Point::from([c.0, c.1]);
}

fn main() -> std::io::Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let opts: Opts = Opts::parse();
    env_logger::init();

    let system = actix::System::new("ride");

    let mut dataset = Dataset::open(Path::new(&opts.gpkg)).unwrap();
    let layer_name = match &opts.layer {
        Some(name) => name.to_string(),
        None => dataset.layer(0).unwrap().name()
    };
    let layer = dataset.layer_by_name(&*layer_name).expect("Layer not found");

    let kph = opts.speed;
    let mps = kph / 3.6;          // meters per second
    let int = opts.interval;      // interval of updates (from sensor)

    for feature in layer.features() {
        let fname = feature.field("name").unwrap().into_string().unwrap();
        let pv: Vec<Point<f64>> = feature.geometry().get_point_vec().into_iter().map(as_point).collect();
        let l: LineString<f64> = LineString::from_iter(pv.iter().map(|p|p.0));

        let d0 = l.geodesic_length();
        debug!("distance: {}m", d0.round());

        let tts = d0 / mps;           // travel time in seconds
        let stp = tts / int as f64;   // steps total
        let ppu = 100.0 / stp;        // percent per update

        // use the factor value to increase the playback speed
        let step_length = actix::clock::Duration::from_millis(int * 1000 / opts.factor);

        let mut wp:Vec<WayPoint> =  Vec::with_capacity(stp as usize);
        for s in 0..=(stp as i64) {
            let sp: Point<f64> = match s {
                0 => pv.first().unwrap().x_y().into(),
                v if v < stp as i64 => {
                    let pct = (s as f64 * ppu) / 100.0;
                    l.line_interpolate_point(&pct).x_y().into()
                },
                _ => pv.last().unwrap().x_y().into()
            };
            wp.push(WayPoint{ id: fname.clone(), pos: sp});
        }

        rt.enter(|| {
            Driver::create(|ctx| {
                Driver::add_stream(throttle(step_length, stream::iter(wp)), ctx);
                Driver {
                    uri: opts.uri.clone(),
                    current_step: 0,
                    total_steps: stp as u64,
                    traveled: 0.0,
                    previous_point: None
                }
            });
        });
    }

    system.run()
}
