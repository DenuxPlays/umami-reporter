use crate::config::Config;
use anyhow::{Context, Result, bail};
use jiff::{Timestamp, ToSpan, civil::Date, tz::TimeZone};
use reqwest::Url;
use serde_json::Value;
use tracing::debug;

pub struct Report {
    pub month: String,
    pub visitors: u64,
    pub visits: u64,
    pub views: u64,
}

pub fn previous_month() -> (Date, Date) {
    let today = Timestamp::now().to_zoned(TimeZone::UTC).date();
    let this_month = Date::new(today.year(), today.month(), 1)
        .expect("the first day of a valid month must be valid");
    let previous = this_month
        .checked_sub(1.month())
        .expect("subtracting one month from a valid date must be valid");

    (previous, this_month)
}

pub async fn fetch(config: &Config, start: Date, end: Date) -> Result<Report> {
    let mut url = Url::parse(&config.umami_url).context("UMAMI_URL ist keine gültige URL")?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("UMAMI_URL darf keine Basis-URL mit Fragment sein"))?
        .extend(["api", "websites", &config.umami_website_id, "stats"]);
    url.query_pairs_mut()
        .append_pair("startAt", &midnight_timestamp(start)?.to_string())
        .append_pair("endAt", &midnight_timestamp(end)?.to_string());

    debug!(start = %start, end = %end, "Frage Umami-Statistiken ab");
    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(&config.umami_api_key)
        .send()
        .await
        .context("Umami konnte nicht erreicht werden")?;
    let status = response.status();
    let body: Value = response.json().await.context("Umami lieferte kein JSON")?;
    if !status.is_success() {
        bail!("Umami antwortete mit {status}: {body}");
    }

    Ok(Report {
        month: format!("{:02}.{:04}", start.month(), start.year()),
        visitors: metric(&body, "visitors")?,
        visits: metric(&body, "visits")?,
        views: metric(&body, "views").or_else(|_| metric(&body, "pageviews"))?,
    })
}

fn midnight_timestamp(date: Date) -> Result<i64> {
    Ok(date
        .at(0, 0, 0, 0)
        .to_zoned(TimeZone::UTC)
        .context("UTC-Datum konnte nicht in einen Zeitstempel umgewandelt werden")?
        .timestamp()
        .as_millisecond())
}

fn metric(body: &Value, name: &str) -> Result<u64> {
    body.get(name)
        .and_then(Value::as_u64)
        .with_context(|| format!("Umami-Antwort enthält keine gültige Kennzahl «{name}»"))
}

#[cfg(test)]
mod tests {
    use super::{metric, previous_month};
    use jiff::{Timestamp, ToSpan, civil::Date, tz::TimeZone};
    use serde_json::json;

    #[test]
    fn previous_month_ends_at_first_day_of_current_month() {
        let (start, end) = previous_month();
        let today = Timestamp::now().to_zoned(TimeZone::UTC).date();
        assert_eq!(end, Date::new(today.year(), today.month(), 1).unwrap());
        assert_eq!(start, end.checked_sub(1_i64.month()).unwrap());
    }

    #[test]
    fn extracts_metrics_and_rejects_invalid_values() {
        let body = json!({"visitors": 12, "views": 34});
        assert_eq!(metric(&body, "visitors").unwrap(), 12);
        assert_eq!(metric(&body, "views").unwrap(), 34);
        assert!(metric(&body, "visits").is_err());
        assert!(metric(&json!({"visitors": -1}), "visitors").is_err());
    }
}
