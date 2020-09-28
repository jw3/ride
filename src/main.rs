use std::path::Path;

use gdal::spatial_ref::SpatialRef;
use gdal::vector::Dataset;
use geo::{LineString, Point};
use geo::algorithm::line_interpolate_point::LineInterpolatePoint;
use geo::prelude::*;

fn as_point(g: &(f64, f64, f64)) -> Point<f64> {
    return Point::from([g.0, g.1]);
}

fn main() {
    let mut dataset = Dataset::open(Path::new(".local/test-ride.gpkg")).unwrap();
    let layer = dataset.layer(0).unwrap();
    let srs = SpatialRef::from_epsg(32617).unwrap();

    for feature in layer.features() {
        let _fname = feature.field("name").unwrap();
        let geometry = feature.geometry().transform_to(&srs).unwrap();
        for pt in geometry.get_point_vec().iter().map(as_point) {
            println!("{}, {}", pt.0.x, pt.0.y);
        }

        let pv = feature.geometry().get_point_vec();
        let p0 = pv.first().map(as_point).unwrap();
        let p1 = pv.last().map(as_point).unwrap();
        let d0 = p0.geodesic_distance(&p1);
        println!("{}m", d0.round());


        let l: LineString<f64> = vec![[p0.0.x, p0.0.y],[p1.0.x, p1.0.y]].into();
        let p2: Point<f64> = l.line_interpolate_point(&0.5).x_y().into();
        let d1 = p0.geodesic_distance(&p2);
        println!("{}m", d1.round());

       assert_eq!(d0 as i64 / 2, d1 as i64);
    }
}
