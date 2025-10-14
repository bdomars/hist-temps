use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
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

    #[clap(long)]
    /// Generate this many random data points instead of fetching from FMI
    random_count: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let tempdata = fmi::Temperatures::new(&args.place);
    let start_time: DateTime<Utc> = args.starttime.parse()?;
    let end_time: DateTime<Utc> = args.endtime.parse()?;

    let data = if let Some(count) = args.random_count {
        let datapoints = generate_random_datapoints(start_time, count);
        println!(
            "Generated {} random datapoints starting at {}",
            datapoints.len(),
            start_time
        );
        datapoints
    } else {
        tempdata.fetch(start_time, end_time).await?
    };

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

fn generate_random_datapoints(start: DateTime<Utc>, count: usize) -> Vec<fmi::Datapoint> {
    use rand::Rng;

    if count == 0 {
        return Vec::new();
    }

    let mut rng = rand::thread_rng();
    (0..count)
        .map(|offset| fmi::Datapoint {
            timestamp: start + Duration::hours(offset as i64),
            value: rng.gen_range(-25.0..35.0),
        })
        .collect()
}
