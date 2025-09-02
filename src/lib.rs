//! UFV Timesheet Utility Library
//!
//! This library provides functionality for extracting timesheet data from Notion,
//! generating PDF timesheets, and sending them via email.

pub mod helpers;
pub mod models;
pub mod service;

pub use service::{TimesheetConfig, TimesheetService};

// Re-export key types for convenience
pub use helpers::pdf::{TimesheetData, TimesheetEntry};
pub use models::notion::{NotionResponse, Page};
