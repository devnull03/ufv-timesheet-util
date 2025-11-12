use pyo3::prelude::*;
use chrono::{DateTime, Datelike};
use std::convert::TryFrom;
use crate::Page;


#[pyclass]
#[derive(Clone)]
pub struct TimesheetData {
    #[pyo3(get)]
    pub entries: Vec<TimesheetEntry>,
    #[pyo3(get)]
    pub total_hours: f64,
}

#[pyclass]
#[derive(Clone)]
pub struct TimesheetEntry {
    #[pyo3(get)]
    pub month: u32,
    #[pyo3(get)]
    pub day: u32,
    #[pyo3(get)]
    pub start: String,
    #[pyo3(get)]
    pub end: String,
    #[pyo3(get)]
    pub paid_hours: f64,
}



impl TryFrom<Page> for TimesheetEntry {
    type Error = String;

    fn try_from(page: Page) -> Result<Self, Self::Error> {
        let start_str = &page.properties.start_and_end.date.start;

        let start_date = DateTime::parse_from_str(start_str, "%Y-%m-%dT%H:%M:%S%.3f%:z")
            .or_else(|_| DateTime::parse_from_str(start_str, "%Y-%m-%dT%H:%M:%S%:z"))
            .or_else(|_| DateTime::parse_from_str(start_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
            .or_else(|_| DateTime::parse_from_str(start_str, "%Y-%m-%dT%H:%M:%SZ"))
            .map_err(|e| format!("Invalid start date format '{}': {}", start_str, e))?;

        let month = start_date.month();
        let day = start_date.day();

        let start = start_date.format("%H:%M").to_string();

        let end = page
            .properties
            .start_and_end
            .date
            .end
            .as_ref()
            .ok_or("Missing end time")?;

        let end_date = DateTime::parse_from_str(end, "%Y-%m-%dT%H:%M:%S%.3f%:z")
            .or_else(|_| DateTime::parse_from_str(end, "%Y-%m-%dT%H:%M:%S%:z"))
            .or_else(|_| DateTime::parse_from_str(end, "%Y-%m-%dT%H:%M:%S.%fZ"))
            .or_else(|_| DateTime::parse_from_str(end, "%Y-%m-%dT%H:%M:%SZ"))
            .map_err(|e| format!("Invalid end date format '{}': {}", end, e))?;

        let end = end_date.format("%H:%M").to_string();

        let paid_hours = page
            .properties
            .billable_hours
            .formula
            .number
            .ok_or("Missing Hours property")?;

        Ok(TimesheetEntry {
            month,
            day,
            start,
            end,
            paid_hours,
        })
    }
}

impl TryFrom<Vec<Page>> for TimesheetData {
    type Error = String;

    fn try_from(pages: Vec<Page>) -> Result<Self, Self::Error> {
        if pages.len() > 16 {
            return Err("Exceeds max entry length 16".to_string());
        }

        let mut entries = Vec::new();
        let mut total_hours: f64 = 0.0;

        for page in pages {
            let entry = TimesheetEntry::try_from(page)?;
            total_hours += entry.paid_hours;
            entries.push(entry);
        }

        Ok(TimesheetData {
            entries,
            total_hours: total_hours.into(),
        })
    }
}

impl TryFrom<Vec<TimesheetEntry>> for TimesheetData {
    type Error = String;

    fn try_from(entries: Vec<TimesheetEntry>) -> Result<Self, Self::Error> {
        if entries.len() > 16 {
            return Err("Exceeds max entry length 16".to_string());
        }

        let mut total_hours = 0.0;

        for entry in &entries {
            total_hours += entry.paid_hours;
        }

        Ok(TimesheetData {
            entries,
            total_hours,
        })
    }
}

