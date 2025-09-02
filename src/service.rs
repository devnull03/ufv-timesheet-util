use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use resend_rs::Resend;
use std::{error::Error, sync::Arc};
use tracing::{error, info};

use crate::{
    helpers::{email, notion, pdf::create_sasi_timesheet},
    models::notion::WebhookAutomationEvent,
    TimesheetData,
};

/// Configuration for the timesheet service
#[derive(Clone)]
pub struct TimesheetConfig {
    pub db_id: String,
    pub automation_id: String,
}

/// The main timesheet service that handles Notion data extraction,
/// PDF generation, and email sending
#[derive(Clone)]
pub struct TimesheetService {
    pub notion_client: Client,
    pub resend: Resend,
    pub config: TimesheetConfig,
}

impl TimesheetService {
    /// Create a new timesheet service instance
    pub fn new(notion_client: Client, resend: Resend, config: TimesheetConfig) -> Self {
        info!("Creating new TimesheetService instance");
        Self {
            notion_client,
            resend,
            config,
        }
    }

    /// Create an Axum router for the timesheet service
    pub fn router(self) -> Router {
        info!("Creating timesheet service router");
        let shared_state = Arc::new(self);
        
        Router::new()
            .route("/timesheet-webhook", post(timesheet_webhook))
            .route("/timesheet-test", get(timesheet_test))
            .route("/timesheet-db-info", get(timesheet_db_info))
            .with_state(shared_state)
    }

    /// Process timesheet data: fetch from Notion, create PDF, send email
    pub async fn process_timesheet(&self) -> Result<String, Box<dyn Error>> {
        info!("Processing timesheet for database: {}", self.config.db_id);
        
        let timesheet_raw_data = notion::fetch_data(&self.notion_client, &self.config.db_id).await?;

        match TimesheetData::try_from(timesheet_raw_data.results) {
            Ok(timesheet_data) => {
                info!(
                    "Successfully parsed timesheet data with {} entries",
                    timesheet_data.entries.len()
                );

                match create_sasi_timesheet(timesheet_data) {
                    Ok(timesheet_pdf) => {
                        info!(
                            "Successfully created timesheet PDF, size: {} bytes",
                            timesheet_pdf.len()
                        );

                        match email::send_timesheet_email(&self.resend, timesheet_pdf).await {
                            Ok(res) => {
                                info!("Email sent successfully with ID: {}", res.id);
                                Ok(res.id.to_string())
                            }
                            Err(e) => {
                                error!("Error sending email: {}", e);
                                let error_msg = format!("Error sending email: {}", e);
                                let _ = email::send_error_info(&self.resend, &error_msg).await;
                                Err(Box::new(e))
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to create timesheet PDF: {}", e);
                        let error_msg = format!("Error creating timesheet PDF: {}", e);
                        let _ = email::send_error_info(&self.resend, &error_msg).await;
                        Err(e.into())
                    }
                }
            }
            Err(err) => {
                error!("Error parsing Notion database: {}", err);
                let error_msg = format!("Error with parsing your linked database: {}", err);
                let _ = email::send_error_info(&self.resend, &error_msg).await;
                Err(err.into())
            }
        }
    }
}

// Route handlers
async fn timesheet_webhook(
    State(service): State<Arc<TimesheetService>>,
    Json(payload): Json<WebhookAutomationEvent>,
) -> String {
    info!("Received timesheet webhook from Notion");

    if payload.source.automation_id != service.config.automation_id {
        info!(
            "Automation ID mismatch. Received: {}, Expected: {}",
            payload.source.automation_id, 
            service.config.automation_id
        );
        return "not the automation you are looking for".to_string();
    }

    match service.process_timesheet().await {
        Ok(email_id) => {
            info!("Timesheet processed successfully, email ID: {}", email_id);
            email_id
        }
        Err(e) => {
            error!("Failed to process timesheet: {}", e);
            format!("Error processing timesheet: {}", e)
        }
    }
}

async fn timesheet_test(State(service): State<Arc<TimesheetService>>) -> String {
    info!("Processing test timesheet request");
    
    match service.process_timesheet().await {
        Ok(email_id) => {
            info!("Test timesheet processed successfully, email ID: {}", email_id);
            format!("Timesheet processed successfully! Email ID: {}", email_id)
        }
        Err(e) => {
            error!("Failed to process test timesheet: {}", e);
            format!("Error processing timesheet: {}", e)
        }
    }
}

async fn timesheet_db_info(State(service): State<Arc<TimesheetService>>) -> String {
    info!("Retrieving database structure for: {}", service.config.db_id);
    
    match notion::retrive_db(&service.notion_client, &service.config.db_id).await {
        Ok(db_info) => db_info,
        Err(e) => {
            error!("Failed to retrieve database info: {}", e);
            format!("Error retrieving database info: {}", e)
        }
    }
}
