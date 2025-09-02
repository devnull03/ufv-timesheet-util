use reqwest::{header, Client};
use std::error::Error;
use tracing::{error, info};

use crate::models;

pub fn notion_client_init(key: String) -> Result<Client, Box<dyn Error>> {
    info!("Initializing Notion client");

    let notion_api_key = match header::HeaderValue::from_str(format!("Bearer {}", key).as_str()) {
        Ok(value) => {
            let mut val = value;
            val.set_sensitive(true);
            val
        }
        Err(e) => {
            error!("Failed to create Authorization header value: {}", e);
            return Err(Box::new(e));
        }
    };

    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, notion_api_key);
    headers.insert(
        "Notion-Version",
        header::HeaderValue::from_static("2022-06-28"),
    );
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    info!("Building Notion client with headers");
    match Client::builder().default_headers(headers).build() {
        Ok(client) => {
            info!("Notion client initialized successfully");
            Ok(client)
        }
        Err(e) => {
            error!("Failed to build Notion client: {}", e);
            Err(Box::new(e))
        }
    }
}

pub async fn fetch_data(
    client: &Client,
    db_id: &String,
) -> Result<models::notion::NotionResponse, Box<dyn Error>> {
    info!("Building filters for database query");
    let filters = utils::build_filters();

    let url = format!("https://api.notion.com/v1/databases/{db_id}/query");
    info!("Fetching data from Notion database: {}", db_id);

    let response = match client.post(&url).body(filters).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let error_text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                error!(
                    "Notion API returned error status {}: {}",
                    status, error_text
                );
                return Err(
                    format!("Notion API returned status {}: {}", status, error_text).into(),
                );
            }
            resp
        }
        Err(e) => {
            error!("Failed to send request to Notion API: {}", e);
            return Err(Box::new(e));
        }
    };

    let text = match response.text().await {
        Ok(text) => {
            info!("Successfully received response from Notion API");
            text
        }
        Err(e) => {
            error!("Failed to read response body: {}", e);
            return Err(Box::new(e));
        }
    };

    match serde_json::from_str::<models::notion::NotionResponse>(&text) {
        Ok(notion_data) => {
            info!(
                "Successfully parsed Notion response with {} results",
                notion_data.results.len()
            );
            Ok(notion_data)
        }
        Err(e) => {
            error!("Failed to parse Notion response: {}", e);
            error!("Raw response: {}", text);
            Err(Box::new(e))
        }
    }
}

pub async fn retrive_db(
    client: &reqwest::Client,
    db_id: &String,
) -> Result<String, Box<dyn Error>> {
    let url = format!("https://api.notion.com/v1/databases/{db_id}/");
    info!("Retrieving database structure from: {}", url);

    let response = match client.get(&url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let error_text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                error!(
                    "Notion API returned error status {}: {}",
                    status, error_text
                );
                return Err(
                    format!("Notion API returned status {}: {}", status, error_text).into(),
                );
            }
            resp
        }
        Err(e) => {
            error!("Failed to send request to Notion API: {}", e);
            return Err(Box::new(e));
        }
    };

    match response.text().await {
        Ok(text) => {
            info!(
                "Successfully retrieved database structure, response length: {} chars",
                text.len()
            );
            Ok(format!("{:?}", &text))
        }
        Err(e) => {
            error!("Failed to read response body: {}", e);
            Err(Box::new(e))
        }
    }
}

pub mod utils {
    use chrono::{Datelike, Local, NaiveDate};
    use tracing::info;

    pub fn get_current_pay_period() -> (NaiveDate, NaiveDate) {
        let mut current_period: (NaiveDate, NaiveDate) =
            (NaiveDate::default(), NaiveDate::default());

        let period_window = (9, 23);
        let now = Local::now().date_naive();
        let day = now.day();

        info!("Calculating pay period for current date: {}", now);

        if day <= period_window.0 {
            current_period.0 = now
                .with_day(period_window.1 + 1)
                .unwrap()
                .with_month(if now.month() == 1 {
                    12
                } else {
                    now.month() - 1
                })
                .unwrap();
            current_period.1 = now.with_day(period_window.0 - 1).unwrap();
            info!(
                "Period calculated (early month): {} to {}",
                current_period.0, current_period.1
            );
        } else if day >= period_window.1 {
            current_period.0 = now.with_day(period_window.1 + 1).unwrap();
            current_period.1 = now
                .with_day(period_window.0 - 1)
                .unwrap()
                .with_month(if now.month() == 12 {
                    1
                } else {
                    now.month() + 1
                })
                .unwrap();
            info!(
                "Period calculated (late month): {} to {}",
                current_period.0, current_period.1
            );
        } else {
            current_period.0 = now.with_day(period_window.0).unwrap();
            current_period.1 = now.with_day(period_window.1).unwrap();
            info!(
                "Period calculated (mid month): {} to {}",
                current_period.0, current_period.1
            );
        }

        current_period
    }

    pub fn build_filters() -> String {
        let date_property_name = "start and end";
        let current_pay_period = get_current_pay_period();

        info!(
            "Building filters for pay period: {} to {}",
            current_pay_period.0, current_pay_period.1
        );

        let filter_string = format!(
            r#"{{"filter": {{"or": [ {{"property": "notes","rich_text": {{"contains": "\\ TODO"}} }},{{"and": [{{"property": "{date_property_name}","date": {{"on_or_after": "{pay_period_start}"}}}},{{"property": "{date_property_name}","date": {{"on_or_before": "{pay_period_end}"}}}} ]}} ]}}, "sorts": [{{"property": "{date_property_name}", "direction": "ascending"}}] }}"#,
            pay_period_start = current_pay_period.0,
            pay_period_end = current_pay_period.1
        );

        info!(
            "Filter string created with length: {} chars",
            filter_string.len()
        );
        filter_string
    }
}
