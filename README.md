# UFV Timesheet Utility Library

A Rust library for automating timesheet processing with Notion integration, PDF generation, and email delivery.

## Features

- **Notion Integration**: Fetch timesheet data from Notion databases
- **PDF Generation**: Create filled PDF timesheets from templates
- **Email Delivery**: Send completed timesheets via email
- **Axum Router**: Ready-to-use HTTP endpoints
- **Error Handling**: Comprehensive error reporting via email

## Quick Start

### Add to Your Project

```toml
[dependencies]
ufv-timesheet-util = { path = "../ufv-timesheet-util" }
```

### Basic Usage

```rust
use ufv_timesheet_util::{
    helpers::notion,
    service::{TimesheetConfig, TimesheetService},
};
use axum::Router;
use reqwest::Client;
use resend_rs::Resend;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize services
    let notion_client = notion::notion_client_init("your-notion-api-key".to_string())?;
    let resend = Resend::new("your-resend-api-key");
    
    let config = TimesheetConfig {
        db_id: "your-database-id".to_string(),
        automation_id: "your-automation-id".to_string(),
    };

    // Create the timesheet service
    let service = TimesheetService::new(notion_client, resend, config);

    // Use as a router
    let app = Router::new().nest("/timesheet", service.router());

    // Or use directly
    let result = service.process_timesheet().await?;
    println!("Timesheet processed: {}", result);

    Ok(())
}
```

## API Endpoints

When using the service as a router, it provides these endpoints:

- `POST /timesheet-webhook` - Handle Notion automation webhooks
- `GET /timesheet-test` - Manually trigger timesheet processing  
- `GET /timesheet-db-info` - Get database structure information

## Configuration

Create a `Secrets.toml` file with your API keys:

```toml
RESEND_API_KEY = 'your-resend-api-key'
TIMESHEET_NOTION_API_KEY = 'your-notion-api-key'
TIMESHEET_DB_ID = 'your-notion-database-id'
TIMESHEET_AUTOMATION_ID = 'your-notion-automation-id'
```

## Dependencies

- **axum**: Web framework for HTTP endpoints
- **reqwest**: HTTP client for Notion API
- **resend-rs**: Email delivery service
- **lopdf**: PDF manipulation
- **chrono**: Date/time handling
- **serde**: JSON serialization

## Templates

The library requires a PDF template file at `templates/sasi.pdf`. This template should contain form fields that match the expected timesheet format.

## Error Handling

The library automatically sends error notifications via email when processing fails, including:

- Notion API errors
- PDF generation failures  
- Email delivery issues
- Data parsing problems

## Examples

See the `examples/` directory for complete usage examples.
