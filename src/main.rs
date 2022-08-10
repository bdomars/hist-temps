use hist_temps::fmi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tempdata = fmi::Temperatures::new("Turku");

    let start_time = "2022-08-01T00:00:00Z".parse()?;
    let end_time = "2022-08-02T00:00:00Z".parse()?;

    let data = tempdata.fetch(start_time, end_time).await?;

    println!("{:?}", data);

    Ok(())
}
