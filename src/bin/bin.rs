use actix::fut::wrap_future;
use actix::prelude::*;
use clap::Clap;
use futures::executor::block_on;
use futures::stream;
use gdal::vector::Feature;
use gdal::Dataset;
use geo::algorithm::line_interpolate_point::LineInterpolatePoint;
use geo::prelude::*;
use geo::{LineString, Point};
use log::{debug, info};
use std::iter::FromIterator;
use std::path::Path;
use tokio::time::throttle;

use libride::cli;
use libride::cli::SubCommand;
use libride::event::{Event, Publisher};

#[derive(Message)]
#[rtype(result = "()")]
struct WayPoint {
    id: String,
    pos: Point<f64>,
}

impl From<WayPoint> for Event {
    fn from(p: WayPoint) -> Self {
        Event {
            id: p.id,
            x: format!("{:.6}", p.pos.x_y().0),
            y: format!("{:.6}", p.pos.x_y().1),
        }
    }
}

#[derive(Clone, Copy)]
struct Travel {
    current_step: u64,
    total_steps: u64,
    traveled: f64,
    previous_point: Option<Point<f64>>,
}

impl Travel {
    fn new(total_steps: u64) -> Self {
        Travel {
            total_steps,
            ..Travel::default()
        }
    }
}

impl Default for Travel {
    fn default() -> Self {
        Travel {
            current_step: 0,
            total_steps: 0,
            traveled: 0.0,
            previous_point: None,
        }
    }
}

#[derive(Clone)]
struct Driver(Publisher, Travel);

impl Actor for Driver {
    type Context = Context<Self>;
}

impl Driver {
    fn move_to(&mut self, p: Point<f64>) {
        let d = self
            .1
            .previous_point
            .map(|prev| prev.geodesic_distance(&p))
            .unwrap_or(0.0);
        self.1.traveled += d;
        self.1.current_step += 1;
        self.1.previous_point = Some(p);
    }

    async fn call(self, e: Event) -> Result<(), String> {
        self.0.publish(e).await
    }
}

impl StreamHandler<WayPoint> for Driver {
    fn handle(&mut self, p: WayPoint, ctx: &mut Context<Self>) {
        let pct = (self.1.current_step as f64 / self.1.total_steps as f64) * 100.0;
        self.move_to(p.pos);

        info!(
            "{}: {:.1}m ({:.1}%)",
            self.1.current_step, self.1.traveled, pct
        );

        let f = self.clone().call(p.into());
        let af =
            wrap_future(f).map(
                move |res, _actor: &mut Self, _ctx: &mut Context<Self>| match res {
                    // todo;; return a result and inspect here
                    _ => (),
                },
            );
        ctx.wait(af);
    }
}

fn as_point(c: (f64, f64, f64)) -> Point<f64> {
    Point::from([c.0, c.1])
}

fn usable_feature(feat: &Feature, did: &str) -> bool {
    !feat.geometry().is_empty()
        && match feat.field(did).expect("device id field not found") {
            None => false,
            Some(id) => !id
                .into_string()
                .expect("device id field not a string")
                .is_empty(),
        }
}

fn main() -> std::io::Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let opts: cli::Opts = cli::Opts::parse();
    env_logger::init();

    let system = actix::System::new("ride");

    let dataset = Dataset::open(Path::new(&opts.gpkg)).unwrap();
    let layer_name = match &opts.layer {
        Some(name) => name.to_string(),
        None => dataset.layer(0).unwrap().name(),
    };
    let mut layer = dataset
        .layer_by_name(&*layer_name)
        .expect("Layer not found");

    let kph = opts.speed;
    let mps = kph / 3.6; // meters per second
    let int = opts.interval; // interval of updates (from sensor)
    let did = opts.did.as_str();

    let output = match &opts.output {
        SubCommand::Stdout(cmd) => block_on(Publisher::stdout(cmd.pretty)),
        SubCommand::Http(cmd) => block_on(Publisher::http(&cmd.uri, cmd.insecure)),
        SubCommand::Mqtt(cmd) => block_on(Publisher::mqtt(&cmd.uri, &cmd.topic)),
    };

    for feature in layer.features().filter(|f| usable_feature(f, did)) {
        let fname = feature.field(did).unwrap().unwrap().into_string().unwrap();
        let pv: Vec<Point<f64>> = feature
            .geometry()
            .get_point_vec()
            .into_iter()
            .map(as_point)
            .collect();
        let l: LineString<f64> = LineString::from_iter(pv.iter().map(|p| p.0));

        let d0 = l.geodesic_length();
        debug!("distance: {}m", d0.round());

        let tts = d0 / mps; // travel time in seconds
        let stp = tts / int as f64; // steps total
        let ppu = 100.0 / stp; // percent per update

        // use the factor value to increase the playback speed
        let step_length = std::time::Duration::from_millis(int * 1000 / opts.factor);

        let mut wp: Vec<WayPoint> = Vec::with_capacity(stp as usize);
        for s in 0..=(stp as i64) {
            let sp: Point<f64> = match s {
                0 => pv.first().unwrap().x_y().into(),
                v if v < stp as i64 => {
                    let pct = (s as f64 * ppu) / 100.0;
                    l.line_interpolate_point(pct).unwrap().x_y().into()
                }
                _ => pv.last().unwrap().x_y().into(),
            };
            wp.push(WayPoint {
                id: fname.clone(),
                pos: sp,
            });
        }

        rt.enter(|| {
            Driver::create(|ctx| {
                Driver::add_stream(throttle(step_length, stream::iter(wp)), ctx);
                Driver(output.clone(), Travel::new(stp as u64))
            });
        });
    }

    system.run()
}
