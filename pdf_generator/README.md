# pdf_generator

A Python module for generating filled SASI timesheet PDFs from structured data.

## Overview

This module provides functionality similar to the Rust implementation in `src/helpers/pdf_.rs`. It takes timesheet data (entries with dates, times, and hours) and fills a PDF template using PyPDFForm.

## Installation

The module uses PyPDFForm for PDF manipulation:

```bash
uv add PyPDFForm
```

## Usage

### Basic Example

```python
from pdf_generator.main import TimesheetData, TimesheetEntry, create_sasi_timesheet

# Create timesheet data
data = TimesheetData(
    entries=[
        TimesheetEntry(month=11, day=1, start="09:00", end="17:00", paid_hours=8.0),
        TimesheetEntry(month=11, day=2, start="09:00", end="17:00", paid_hours=8.0),
    ],
    total_hours=16.0
)

# Generate PDF
pdf_bytes = create_sasi_timesheet(data, template_path="../templates/sasi.pdf")

# Save to file
with open("output.pdf", "wb") as f:
    f.write(pdf_bytes)
```

### Data Structures

#### `TimesheetEntry`
Represents a single work entry:
- `month` (int): Month number (1-12)
- `day` (int): Day of month
- `start` (str): Start time in "HH:MM" format
- `end` (str): End time in "HH:MM" format
- `paid_hours` (float): Hours to be paid for this entry

#### `TimesheetData`
Container for all timesheet information:
- `entries` (List[TimesheetEntry]): List of timesheet entries (max 16)
- `total_hours` (float): Total hours across all entries

### API Reference

#### `create_sasi_timesheet(data: TimesheetData, template_path: str = "../templates/sasi.pdf") -> bytes`

Fills the SASI timesheet PDF template with provided data.

**Parameters:**
- `data`: TimesheetData containing entries and total hours
- `template_path`: Path to the PDF template file (default: "../templates/sasi.pdf")

**Returns:**
- `bytes`: The filled PDF as bytes

**Raises:**
- `ValueError`: If data contains more than 16 entries
- `FileNotFoundError`: If template file doesn't exist

## Utility Scripts

### inspect_form.py

Use this script to inspect the form fields in your PDF template:

```bash
cd pdf_generator
python inspect_form.py
```

This will list all available form fields in the template, which is useful for debugging or adapting to different PDF templates.

## Differences from Rust Implementation

The Python implementation mirrors the Rust version but with some differences:

1. **Library**: Uses PyPDFForm instead of lopdf
2. **Error Handling**: Uses Python exceptions instead of Result types
3. **Type System**: Uses dataclasses instead of structs
4. **Simplicity**: PyPDFForm handles appearance streams automatically, so no manual stream creation needed

## Field Name Mapping

The module maps data to PDF form fields using these prefixes and patterns:
- `"Month DayRow{n}"` - Month field for row n
- `"Month DayRow{n}_2"` - Day field for row n
- `"Start TimeRow{n}"` - Start time for row n
- `"Finish TimeRow{n}"` - End time for row n
- `"Hours to be PaidRow{n}"` - Paid hours for row n
- `"Total hoursRow1"` - Total hours field

Where `{n}` is the row number from 1 to 16.

Example field names:
- Month: `"Month DayRow1"` → "11"
- Day: `"Month DayRow1_2"` → "15"
- Start: `"Start TimeRow1"` → "09:00"
- Finish: `"Finish TimeRow1"` → "17:00"
- Hours: `"Hours to be PaidRow1"` → "8.0"

## Limitations

- Maximum 16 entries per timesheet (same as Rust implementation)
- Template must exist at specified path
- Field names must match expected format
