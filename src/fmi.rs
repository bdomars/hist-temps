use chrono::prelude::*;
use regex::Regex;

#[derive(Debug)]
pub struct Datapoint {
    timestamp: DateTime<Utc>,
    value: f64,
}

pub struct Temperatures {
    client: reqwest::Client,
}

const FMI_URL: &str = "https://opendata.fmi.fi/wfs";

impl Temperatures {
    pub fn new() -> Temperatures {
        Temperatures {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Datapoint>, Box<dyn std::error::Error>> {
        let params = [
            ("starttime", start_time.to_rfc3339()),
            ("endtime", end_time.to_rfc3339()),
            ("service", "WFS".to_string()),
            ("version", "2.0.0".to_string()),
            ("request", "getFeature".to_string()),
            (
                "storedquery_id",
                "fmi::observations::weather::hourly::timevaluepair".to_string(),
            ),
            ("parameters", "temperature".to_string()),
            ("place", "Turku".to_string()),
        ];

        let req = self.client.get(FMI_URL).query(&params);
        let resp = req.send().await?.text().await?;

        let tvp_regex = Regex::new(r"<wml2:MeasurementTVP>\s*?<wml2:time>(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z)</wml2:time>\s*?<wml2:value>(\d+\.\d)</wml2:value>\s*?</wml2:MeasurementTVP>").unwrap();
        let mut res = Vec::new();

        for cap in tvp_regex.captures_iter(resp.as_str()) {
            res.push(Datapoint {
                timestamp: cap[1].parse()?,
                value: cap[2].parse()?,
            })
        }

        Ok(res)
    }
}
