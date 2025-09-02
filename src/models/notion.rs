use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookAutomationEvent {
    pub source: AutomationSource,
    pub data: serde_json::Value, // Using generic Value, don't really need this shit
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AutomationSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub automation_id: String,
    pub action_id: String,
    pub event_id: Option<String>,
    pub user_id: Option<String>,
    pub attempt: Option<i32>,
}

// Response structs for Notion API
#[derive(Serialize, Deserialize, Debug)]
pub struct NotionResponse {
    object: String,
    pub results: Vec<Page>,
    next_cursor: Option<String>,
    has_more: bool,
}

impl fmt::Display for NotionResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Notion Response:")?;
        writeln!(f, "  Object Type: {}", self.object)?;
        writeln!(f, "  Has More: {}", self.has_more)?;
        writeln!(f, "  Next Cursor: {:?}", self.next_cursor)?;
        writeln!(f, "  Results Count: {}", self.results.len())?;

        for (i, page) in self.results.iter().enumerate() {
            writeln!(f, "\n=========== Page #{} ===========", i + 1)?;
            writeln!(f, "  ID: {}", page.id)?;
            writeln!(f, "  Object Type: {}", page.object)?;
            writeln!(f, "  URL: {}", page.url)?;
            writeln!(f, "  Created: {}", page.created_time)?;
            writeln!(f, "  Last Edited: {}", page.last_edited_time)?;

            writeln!(f, "\n  Properties:")?;

            // Start and End Date
            writeln!(
                f,
                "    Start and End (ID: {}):",
                page.properties.start_and_end.id
            )?;
            writeln!(
                f,
                "      Type: {}",
                page.properties.start_and_end.property_type
            )?;
            writeln!(
                f,
                "      Start: {}",
                page.properties.start_and_end.date.start
            )?;
            writeln!(f, "      End: {:?}", page.properties.start_and_end.date.end)?;
            writeln!(
                f,
                "      Timezone: {:?}",
                page.properties.start_and_end.date.time_zone
            )?;

            // Billable Hours
            writeln!(
                f,
                "    Billable Hours (ID: {}):",
                page.properties.billable_hours.id
            )?;
            writeln!(
                f,
                "      Type: {}",
                page.properties.billable_hours.property_type
            )?;
            writeln!(
                f,
                "      Formula Type: {}",
                page.properties.billable_hours.formula.value_type
            )?;
            writeln!(
                f,
                "      Hours: {:?}",
                page.properties.billable_hours.formula.number
            )?;

            // Workplace
            writeln!(f, "    Workplace (ID: {}):", page.properties.workplace.id)?;
            writeln!(f, "      Type: {}", page.properties.workplace.property_type)?;
            writeln!(
                f,
                "      Select ID: {}",
                page.properties.workplace.select.id
            )?;
            writeln!(f, "      Name: {}", page.properties.workplace.select.name)?;
            writeln!(f, "      Color: {}", page.properties.workplace.select.color)?;

            // Duration
            writeln!(f, "    Duration (ID: {}):", page.properties.duration.id)?;
            writeln!(f, "      Type: {}", page.properties.duration.property_type)?;
            writeln!(
                f,
                "      Formula Type: {}",
                page.properties.duration.formula.value_type
            )?;
            writeln!(
                f,
                "      Value: {:?}",
                page.properties.duration.formula.number
            )?;

            // Notes
            writeln!(f, "    Notes (ID: {}):", page.properties.notes.id)?;
            writeln!(f, "      Type: {}", page.properties.notes.property_type)?;
            writeln!(
                f,
                "      Text Count: {}",
                page.properties.notes.rich_text.len()
            )?;

            for (j, text) in page.properties.notes.rich_text.iter().enumerate() {
                writeln!(f, "      Text #{}", j + 1)?;
                writeln!(f, "        Type: {}", text.text_type)?;
                writeln!(f, "        Content: {}", text.text.content)?;
                writeln!(f, "        Plain Text: {}", text.plain_text)?;
                writeln!(f, "        Href: {:?}", text.href)?;
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    object: String,
    id: String,
    created_time: String,
    last_edited_time: String,
    pub properties: PageProperties,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PageProperties {
    #[serde(rename = "start and end")]
    pub start_and_end: DateProperty,
    #[serde(rename = "Billable Hours")]
    pub billable_hours: FormulaProperty,
    #[serde(rename = "Workplace")]
    workplace: SelectProperty,
    #[serde(rename = "Duration")]
    duration: FormulaProperty,
    #[serde(rename = "notes")]
    notes: RichTextProperty,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DateProperty {
    id: String,
    #[serde(rename = "type")]
    property_type: String,
    pub date: DateValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DateValue {
    pub start: String,
    pub end: Option<String>,
    time_zone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FormulaProperty {
    id: String,
    #[serde(rename = "type")]
    property_type: String,
    pub formula: FormulaValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FormulaValue {
    #[serde(rename = "type")]
    value_type: String,
    pub number: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SelectProperty {
    id: String,
    #[serde(rename = "type")]
    property_type: String,
    select: SelectValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SelectValue {
    id: String,
    name: String,
    color: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RichTextProperty {
    id: String,
    #[serde(rename = "type")]
    property_type: String,
    rich_text: Vec<RichTextValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RichTextValue {
    #[serde(rename = "type")]
    text_type: String,
    text: TextContent,
    annotations: Option<serde_json::Value>,
    plain_text: String,
    href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextContent {
    content: String,
    link: Option<serde_json::Value>,
}
