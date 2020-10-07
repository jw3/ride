ride along
===

Simulate a sensor riding along a path by interpolating points into a polyline based on travel speed.

### usage

- dump to stdout and pipe; `$ ride test.gpkg | jq .`
- post to uri of event sink; `$ ride test.gpkg --uri http://localhost:9000/api/events | jq .`
- and more...

```
$ ride --help
ride v0.2.0
Simulated sensor riding along geo features

USAGE:
    ride [OPTIONS] <gpkg>

ARGS:
    <gpkg>    GeoPackage containing vector data

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --factor <factor>        simulation playback speed factor [default: 1]
    -i, --interval <interval>    simulated seconds between sensor updates [default: 2]
    -l, --layer <layer>          name of layer to select features from
    -s, --speed <speed>          sensor travel time in kilometers per hour [default: 10.0]
    -u, --uri <uri>              uri to POST events to
```

### ref
- https://gis.stackexchange.com/a/8674
