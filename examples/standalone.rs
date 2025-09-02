use axum::Router;
use reqwest::Client;
use resend_rs::Resend;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use ufv_timesheet_util::{
    helpers::notion,
    service::{TimesheetConfig, TimesheetService},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();

    info!("Starting UFV Timesheet Service example");

    // Load configuration (in a real app, you'd load from environment or config file)
    let notion_api_key = "your-notion-api-key".to_string();
    let resend_api_key = "your-resend-api-key";
    let db_id = "your-database-id".to_string();
    let automation_id = "your-automation-id".to_string();

    // Initialize services
    let notion_client = notion::notion_client_init(notion_api_key)?;
    let resend = Resend::new(resend_api_key);

    let config = TimesheetConfig {
        db_id,
        automation_id,
    };

    // Create the timesheet service
    let timesheet_service = TimesheetService::new(notion_client, resend, config);

    // Create router with the timesheet service
    let app = Router::new()
        .nest("/api/timesheet", timesheet_service.router())
        .route("/health", axum::routing::get(|| async { "OK" }));

    // Start server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

/*
Example usage of the service:

1. POST /api/timesheet/timesheet-webhook
   - Handles Notion webhook events
   - Automatically processes timesheets when triggered

2. GET /api/timesheet/timesheet-test
   - Manual trigger for testing
   - Processes current timesheet data

3. GET /api/timesheet/timesheet-db-info
   - Returns database structure information

The service will:
- Fetch timesheet data from your Notion database
- Convert it to a PDF timesheet
- Email the PDF to the configured recipients
*/
