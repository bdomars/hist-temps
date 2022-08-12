use clap::Parser;
use hist_temps::fmi;

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
    let tempdata = fmi::Temperatures::new(&args.place);
    let start_time = args.starttime.parse()?;
    let end_time = args.endtime.parse()?;
    let data = tempdata.fetch(start_time, end_time).await?;

    if args.write_influxdb {
        use influxdb2::models::DataPoint;
        use influxdb2::Client;
        use tokio_stream as stream;

        let host = std::env::var("INFLUXDB_HOST").expect("env variable INFLUXDB_HOST");
        let org = std::env::var("INFLUXDB_ORG").expect("env variable INFLUXDB_ORG");
        let token = std::env::var("INFLUXDB_TOKEN").expect("env variable INFLUXDB_TOKEN");
        let bucket = "fmi";
        let client = Client::new(host, org, token);

        let points = data.into_iter().map(|dp| {
            DataPoint::builder("measurement")
                .tag("place", "Turku")
                .field("temperature", dp.value)
                .timestamp(dp.timestamp.timestamp_nanos())
                .build()
                .unwrap()
        });

        client.write(bucket, stream::iter(points)).await?;
    } else {
        println!("Got data:\n\n{:?}", data);
    }

    Ok(())
}
