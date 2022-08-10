use hist_temps::fmi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tempdata = fmi::Temperatures::new("Turku");

    let start_time = "2022-08-01T00:00:00Z".parse()?;
    let end_time = "2022-08-02T00:00:00Z".parse()?;

    let data = tempdata.fetch(start_time, end_time).await?;

    use influxdb2::models::DataPoint;
    use influxdb2::Client;
    use tokio_stream::{self as stream, StreamExt};

    let host = std::env::var("INFLUXDB_HOST").expect("env variable INFLUXDB_HOST");
    let org = std::env::var("INFLUXDB_ORG").expect("env variable INFLUXDB_ORG");
    let token = std::env::var("INFLUXDB_TOKEN").expect("env variable INFLUXDB_TOKEN");
    let bucket = "fmi";
    let client = Client::new(host, org, token);

    let points: Vec<DataPoint> = data
        .into_iter()
        .map(|dp| {
            DataPoint::builder("measurement")
                .tag("place", "Turku")
                .field("temperature", dp.value)
                .timestamp(dp.timestamp.timestamp_nanos())
                .build()
                .unwrap()
        })
        .collect();

    println!("{:?}", points);
    client.write(bucket, stream::iter(points)).await?;

    Ok(())
}
