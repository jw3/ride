use clap::Clap;

/// Simulated sensor riding along geo features.
#[derive(Clap)]
#[clap(version = "v0.3.0")]
pub struct Opts {
    /// GeoPackage containing vector data
    pub gpkg: String,

    /// name of layer to select features from
    #[clap(short, long)]
    pub layer: Option<String>,

    /// device id field name
    #[clap(long, default_value = "name")]
    pub did: String,

    /// simulation playback speed factor
    #[clap(short, long, default_value = "1")]
    pub factor: u64,

    /// sensor travel time in kilometers per hour
    #[clap(short, long, default_value = "10.0")]
    pub speed: f64,

    /// simulated seconds between sensor updates
    #[clap(short, long, default_value = "2")]
    pub interval: u64,

    #[clap(subcommand)]
    pub output: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    Stdout(StdoutOpts),
    Http(HttpOpts),
}

#[derive(Clap)]
pub struct StdoutOpts {
    /// pretty formatting of json (both in request and logs)
    #[clap(long)]
    pub pretty: bool,
}

#[derive(Clap)]
pub struct HttpOpts {
    /// uri to POST events to
    #[clap(short, long, default_value = "localhost:9000")]
    pub uri: String,

    /// Controls the use of certificate validation.
    ///
    /// Defaults to `false`.
    ///
    /// # Warning
    ///
    /// You should think very carefully before using this method. If
    /// invalid certificates are trusted, *any* certificate for *any* site
    /// will be trusted for use. This includes expired certificates. This
    /// introduces significant vulnerabilities, and should only be used
    /// as a last resort.
    ///
    /// # Optional
    ///
    /// This requires the optional `default-tls`, `native-tls`, or `rustls-tls`
    /// feature to be enabled.
    #[clap(long)]
    pub insecure: bool,
}
