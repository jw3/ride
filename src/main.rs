use std::thread;
use std::time;
use std::path::Path;
use std::error::Error;

use gdal::spatial_ref::SpatialRef;
use gdal::vector::Dataset;
use gdal::vector::Geometry;
use gdal::vector::OGRwkbGeometryType::wkbPoint;
use geo::{LineString, Point};
use geo::algorithm::line_interpolate_point::LineInterpolatePoint;
use geo::prelude::*;
use serde::Serialize;

use std::iter::FromIterator;

use reqwest::Client;
use reqwest::StatusCode;

use std::collections::HashMap;

#[derive(Serialize)]
struct Event {
    id: String,
    lon: String,
    lat: String,
}

fn as_point(c: (f64, f64, f64)) -> Point<f64> {
    return Point::from([c.0, c.1]);
}

// todo;; get gdal::vector::ToGdal working
fn gdal_of(sp: &Point<f64>) -> Geometry {
    let mut geom = Geometry::empty(wkbPoint).unwrap();
    geom.set_point_2d(0, sp.x_y());
    return geom;
}

fn event(sp: &Point<f64>, uri: &String) -> Result<(), reqwest::Error> {
    let e = Event {
        id: String::from("foo"),
        lon: sp.x_y().0.to_string(),
        lat: sp.x_y().1.to_string()
    };

    let json = serde_json::to_string_pretty(&e).unwrap();
    println!("{}", json);
    let r = reqwest::blocking::Client::new().post(uri).json(&e).send();
    if let Err(e) = r {
        println!("{}", e);
    }
    Ok(())
}

fn main() {
    let mut dataset = Dataset::open(Path::new(".local/test-ride.gpkg")).unwrap();
    let layer = dataset.layer(0).unwrap();

    for feature in layer.features() {
        let pv: Vec<Point<f64>> = feature.geometry().get_point_vec().into_iter().map(as_point).collect();
        let p0 = pv.first().unwrap();
        let p1 = pv.last().unwrap();
        let l: LineString<f64> = LineString::from_iter(pv.iter().map(|p|p.0));
        let d0 = l.geodesic_length();
        println!("{}m", d0.round());

        let kph = 20.0;
        let mps = kph / 3.6;
        let tts = d0 / mps;
        let int = 2;
        let fac = 1;
        let stp = tts / int as f64;
        let pp = 100.0 / stp;
        let step_length = time::Duration::from_millis(int * 1000 / fac);

        let uri = String::from("http://localhost:9000/api/device/move");

        println!("{}: {}m ({}%)", 0, 0.0, 0);
        event(&p0, &uri);
        thread::sleep(step_length);
        let mut traveled = 0.0;
        let mut previous = Point::new(p0.x_y().0, p0.x_y().1);
        for s in 1..(stp as i64) {
            let p = s as f64 * pp / 100.0;
            let sp: Point<f64> = l.line_interpolate_point(&p).x_y().into();
            traveled += previous.geodesic_distance(&sp);
            previous = Point::new(sp.x_y().0, sp.x_y().1);
            println!("{}: {:.1}m ({:.0}%)", s, traveled, p * 100.0);
            event(&sp, &uri);
            thread::sleep(step_length);
        }
        println!("{}: {:.1}m ({}%)", 5, d0, 100);
        event(&p1, &uri);
        thread::sleep(step_length);
    }
}
