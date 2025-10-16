use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug)]
pub struct Datapoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

pub struct Temperatures {
    place: String,
    wfs: WfsClient,
}

const FMI_URL: &str = "https://opendata.fmi.fi/wfs";
const TEMPERATURE_TIMESERIES_HINTS: &[&str] =
    &["-t2m", "-temperature", "TA_PT1H_AVG", "AirTemperature"];

impl Temperatures {
    pub fn new(place: &str) -> Temperatures {
        Temperatures {
            place: place.to_string(),
            wfs: WfsClient::new(reqwest::Client::new(), FMI_URL),
        }
    }

    pub async fn fetch(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Datapoint>> {
        let parameters = vec![
            ("starttime".to_string(), start_time.to_rfc3339()),
            ("endtime".to_string(), end_time.to_rfc3339()),
            ("place".to_string(), self.place.clone()),
        ];

        // Attempt request without explicit parameter filtering; FMI now rejects legacy `parameters=temperature`.
        let response_primary = self
            .wfs
            .get_feature(
                "fmi::observations::weather::hourly::timevaluepair",
                &parameters,
            )
            .await;

        let response = response_primary;

        if let Ok(xml) = &response {
            if let Err(err) = tokio::fs::write("fmi_initial_response.xml", xml).await {
                eprintln!("failed to dump initial FMI response: {err}");
            }
        }

        let xml = response?;
        parse_temperature_timeseries(&xml)
    }
}

#[derive(Clone)]
struct WfsClient {
    client: reqwest::Client,
    base_url: String,
}

impl WfsClient {
    fn new(client: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }

    async fn get_feature(
        &self,
        stored_query_id: &str,
        parameters: &[(String, String)],
    ) -> Result<String> {
        let mut query = vec![
            ("service".to_owned(), "WFS".to_owned()),
            ("version".to_owned(), "2.0.0".to_owned()),
            ("request".to_owned(), "GetFeature".to_owned()),
            ("storedquery_id".to_owned(), stored_query_id.to_owned()),
        ];
        query.extend(parameters.iter().cloned());

        let response = self
            .client
            .get(&self.base_url)
            .query(&query)
            .send()
            .await
            .context("failed to call FMI WFS GetFeature")?
            .error_for_status()
            .context("FMI WFS returned an error status")?
            .text()
            .await
            .context("failed to read FMI WFS response body")?;

        if response.contains("ExceptionReport") {
            let preview = response.lines().take(10).collect::<Vec<_>>().join("\n");
            bail!("FMI WFS exception response: {preview}");
        }

        Ok(response)
    }
}

fn parse_temperature_timeseries(xml: &str) -> Result<Vec<Datapoint>> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut datapoints = Vec::new();
    let mut in_temperature_series = false;
    let mut current_time: Option<DateTime<Utc>> = None;
    let mut upcoming_temperature_series = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                b"MeasurementTimeseries" => {
                    let hint_match = is_temperature_timeseries(e.attributes());
                    in_temperature_series = hint_match || upcoming_temperature_series;
                    upcoming_temperature_series = false;
                }
                b"MeasurementTVP" => {
                    current_time = None;
                }
                b"time" if in_temperature_series => {
                    let text = reader
                        .read_text(e.name())
                        .context("failed to read time value from FMI WFS response")?;
                    let timestamp = DateTime::parse_from_rfc3339(text.trim())
                        .with_context(|| format!("invalid timestamp in FMI WFS response: {text}"))?
                        .with_timezone(&Utc);
                    current_time = Some(timestamp);
                }
                b"value" if in_temperature_series => {
                    let text = reader
                        .read_text(e.name())
                        .context("failed to read measurement value from FMI WFS response")?;
                    let trimmed = text.trim();
                    if trimmed.eq_ignore_ascii_case("nan") || trimmed.is_empty() {
                        continue;
                    }
                    let value: f64 = trimmed.parse().with_context(|| {
                        format!("invalid temperature value in FMI WFS response: {trimmed}")
                    })?;

                    if let Some(timestamp) = current_time {
                        datapoints.push(Datapoint { timestamp, value });
                    }
                }
                b"observedProperty" => {
                    if let Ok(text) = reader.read_text(e.name()) {
                        let needle = text.trim().to_lowercase();
                        upcoming_temperature_series =
                            needle.contains("temperature") || needle.contains("airtemp");
                    }
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                b"MeasurementTimeseries" => {
                    in_temperature_series = false;
                    upcoming_temperature_series = false;
                }
                b"MeasurementTVP" => {
                    current_time = None;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(err) => {
                return Err(anyhow::Error::new(err).context("error while parsing FMI WFS XML"));
            }
        }

        buf.clear();
    }

    Ok(datapoints)
}

fn is_temperature_timeseries(mut attrs: Attributes<'_>) -> bool {
    while let Some(Ok(attr)) = attrs.next() {
        if attr.key.local_name().as_ref() == b"id" || attr.key.as_ref() == b"gml:id" {
            if let Ok(value) = attr.unescape_value() {
                let id = value.as_ref();
                if TEMPERATURE_TIMESERIES_HINTS
                    .iter()
                    .any(|hint| id.contains(hint))
                {
                    return true;
                }
            }
        }
    }

    false
}
