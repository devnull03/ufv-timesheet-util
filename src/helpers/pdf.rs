use chrono::{DateTime, Datelike};
use lopdf::{dictionary, Document, Object, StringFormat};
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{error, info};

use crate::models::notion::Page;

fn load_pdf<P: AsRef<Path>>(path: P) -> Result<Document, lopdf::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Document::load_from(reader)
}

pub fn create_sasi_timesheet(data: TimesheetData) -> Result<Vec<u8>, String> {
    let template_path = "templates/sasi.pdf";
    let mut output_buffer: Vec<u8> = Vec::new();

    let field_identifiers = (
        "Month Day",
        "Start Time",
        "Finish Time",
        "Hours to be Paid",
        "Total hours",
    );

    match load_pdf(template_path) {
        Ok(mut doc) => {
            info!("Loaded PDF with {} page(s)", doc.get_pages().len());

            let field_refs = {
                let catalog = doc.catalog().unwrap();
                let acroform_ref = catalog.get(b"AcroForm").unwrap().as_reference().unwrap();
                let acroform = doc.get_dictionary(acroform_ref).unwrap();

                if let Ok(Object::Array(fields)) = acroform.get(b"Fields") {
                    info!("Found {} form fields", fields.len());

                    fields
                        .iter()
                        .map(|field_ref| field_ref.as_reference().unwrap())
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                }
            };

            let mut processed_entries = 0;

            for field_ref in field_refs.iter() {
                if let Ok(field_dict) = doc.get_dictionary_mut(*field_ref) {
                    if let Ok(Object::String(name_bytes, _)) = field_dict.get(b"T") {
                        let field_name = String::from_utf8_lossy(name_bytes.as_slice());
                        info!("Processing form field: {}", field_name);

                        if field_name.starts_with(field_identifiers.4) {
                            let value = data.total_hours.to_string();
                            update_field_appearance(field_dict, &value);
                            break;
                        }

                        if processed_entries >= data.entries.len() {
                            continue;
                        }

                        let mut value = String::new();

                        match field_name {
                            _ if field_name.starts_with(field_identifiers.0) => {
                                if field_name.ends_with("_2") {
                                    value = data.entries[processed_entries].day.to_string();
                                } else {
                                    value = data.entries[processed_entries].month.to_string();
                                }
                            }
                            _ if field_name.starts_with(field_identifiers.1) => {
                                value = data.entries[processed_entries].start.clone();
                            }
                            _ if field_name.starts_with(field_identifiers.2) => {
                                value = data.entries[processed_entries].end.clone();
                            }
                            _ if field_name.starts_with(field_identifiers.3) => {
                                value = data.entries[processed_entries].paid_hours.to_string();
                                processed_entries += 1
                            }

                            std::borrow::Cow::Borrowed(_) => {}
                            std::borrow::Cow::Owned(_) => {}
                        }

                        if !value.is_empty() {
                            update_field_appearance(field_dict, &value);
                        }
                    }
                }
            }

            match doc.save_to(&mut output_buffer) {
                Ok(_) => info!(
                    "Successfully converted PDF to bytes, size: {} bytes",
                    output_buffer.len()
                ),
                Err(e) => error!("Failed to convert PDF to bytes: {}", e),
            }

            Ok(output_buffer)
        }
        Err(e) => {
            error!("Failed to load PDF: {}", e);
            Err(format!("Failed to load PDF: {:?}", e))
        }
    }
}

fn update_field_appearance(field_dict: &mut lopdf::Dictionary, value: &str) {
    // First: Set the value directly for browser compatibility
    field_dict.set(
        b"V",
        Object::String(value.as_bytes().to_vec(), StringFormat::Literal),
    );

    // Remove any existing appearance stream to start clean
    field_dict.remove(b"AP");

    // Get field rectangle
    let rect = if let Ok(Object::Array(rect)) = field_dict.get(b"Rect") {
        rect.clone()
    } else {
        vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(100),
            Object::Integer(30),
        ]
    };

    // Extract dimensions
    let x1 = if let Object::Integer(val) = rect[0] {
        val as f32
    } else {
        0.0
    };
    let y1 = if let Object::Integer(val) = rect[1] {
        val as f32
    } else {
        0.0
    };
    let x2 = if let Object::Integer(val) = rect[2] {
        val as f32
    } else {
        100.0
    };
    let y2 = if let Object::Integer(val) = rect[3] {
        val as f32
    } else {
        30.0
    };
    let width = x2 - x1;
    let height = y2 - y1;

    // Create a simple but effective appearance stream
    // Using a simpler content stream focused on compatibility
    let stream_content = format!(
        "BT\n/Helv 10 Tf\n0 g\n2 {} Td\n({}) Tj\nET",
        height - 12.0, // Adjust text position for visibility
        value.replace("(", "\\(").replace(")", "\\)")  // Escape parentheses
    );

    // Create a minimal stream dictionary - fewer entries for better compatibility
    let stream_dict = dictionary! {
        b"Subtype" => Object::Name(b"Form".to_vec()),
        b"BBox" => Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(width as i64),
            Object::Integer(height as i64),
        ]),
        b"Resources" => dictionary! {
            b"Font" => dictionary! {
                b"Helv" => dictionary! {
                    b"Type" => Object::Name(b"Font".to_vec()),
                    b"Subtype" => Object::Name(b"Type1".to_vec()),
                    b"BaseFont" => Object::Name(b"Helvetica".to_vec()),
                },
            },
        },
    };

    // Create the stream
    let ap_stream = lopdf::Stream::new(stream_dict, stream_content.as_bytes().to_vec());

    // Set up the appearance dictionary
    let mut ap_dict = dictionary! {};
    ap_dict.set(b"N", Object::Stream(ap_stream));
    field_dict.set(b"AP", Object::Dictionary(ap_dict));

    // Ensure field is set to display as intended
    // For text fields, we want to control these key attributes:
    field_dict.set(b"FT", Object::Name(b"Tx".to_vec())); // Ensure it's a text field
    field_dict.set(
        b"DA",
        Object::String(b"/Helv 10 Tf 0 g".to_vec(), StringFormat::Literal),
    ); // Default appearance

    // MK dictionary for presentation characteristics
    let mut mk_dict = dictionary! {};
    mk_dict.set(
        b"BG",
        Object::Array(vec![
            Object::Real(1.0), // White background
            Object::Real(1.0),
            Object::Real(1.0),
        ]),
    );
    field_dict.set(b"MK", Object::Dictionary(mk_dict));

    // Clear any flags that might hide the field
    // Setting common bits for visibility and printing
    field_dict.set(b"F", Object::Integer(4)); // Print bit set, others cleared

    // Set specific field flags
    let field_flags = 0; // No special flags, basic text field
    field_dict.set(b"Ff", Object::Integer(field_flags));
}

pub struct TimesheetData {
    pub entries: Vec<TimesheetEntry>,
    pub total_hours: f64,
}

pub struct TimesheetEntry {
    pub month: u32,
    pub day: u32,
    pub start: String,
    pub end: String,
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
