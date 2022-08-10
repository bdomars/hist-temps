use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
/// A program for importing historical temperature data from the
/// Finnish Meteorological Institutions WFS service to InfluxDB
struct Args {
    #[clap(short, long)]
    /// Write data to influx
    write_influxdb: bool,

    #[clap(short, long)]
    /// A place name that is passed to the WFS endpoint (eg. a city)
    place: String,

    #[clap(short, long)]
    /// Timestamp in ISO 8601
    starttime: String,

    #[clap(short, long)]
    /// Timestamp in ISO 8601
    endtime: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("{:?}", args);
    Ok(())
}
