from dataclasses import dataclass
from pathlib import Path
from typing import List
from PyPDFForm import PdfWrapper

@dataclass
class TimesheetEntry:
    """Represents a single timesheet entry."""

    month: int
    day: int
    start: str  # Format: "HH:MM"
    end: str  # Format: "HH:MM"
    paid_hours: float


@dataclass
class TimesheetData:
    """Container for all timesheet entries and total hours."""

    entries: List[TimesheetEntry]
    total_hours: float


def create_sasi_timesheet(
    data: TimesheetData,
    template_path: str = "../templates/sasi.pdf",
    output_path: str = "../output.pdf",
) -> str:
    """
    Fill the SASI timesheet PDF template with the provided data.

    Args:
        data: TimesheetData containing entries and total hours
        template_path: Path to the PDF template file

    Returns:
        str: Path to processed pdf
        None: failure

    Raises:
        ValueError: If data contains more than 16 entries
        FileNotFoundError: If template file doesn't exist
    """

    if len(data.entries) > 16:
        raise ValueError("Exceeds max entry length 16")

    field_identifiers = {
        "month_day": "Month Day",
        "start_time": "Start Time",
        "finish_time": "Finish Time",
        "hours_to_be_paid": "Hours to be Paid",
        "total_hours": "Total hours",
    }

    fill_data = {}

    for idx, entry in enumerate(data.entries, start=1):
        month_field = f"{field_identifiers['month_day']}Row{idx}"
        day_field = f"{field_identifiers['month_day']}Row{idx}_2"
        start_field = f"{field_identifiers['start_time']}Row{idx}"
        finish_field = f"{field_identifiers['finish_time']}Row{idx}"
        hours_field = f"{field_identifiers['hours_to_be_paid']}Row{idx}"

        fill_data[month_field] = str(entry.month)
        fill_data[day_field] = str(entry.day)
        fill_data[start_field] = entry.start
        fill_data[finish_field] = entry.end
        fill_data[hours_field] = str(entry.paid_hours)

    total_hours_field = f"{field_identifiers['total_hours']}Row1"
    fill_data[total_hours_field] = str(data.total_hours)

    pdf = PdfWrapper(template_path, adobe_mode=True)
    pdf_bytes = pdf.fill(fill_data).read()

    try:

        Path('/'.join(output_path.split('/')[:-1])).mkdir(parents=True, exist_ok=True)

        with open(output_path, "wb") as f:
            f.write(pdf_bytes)

        print(f"Successfully generated PDF with {len(pdf_bytes)} bytes")
        return output_path
    except Exception as e:
        print(f"Error generating PDF: {e}")
        raise e


def main():
    """Example usage of the timesheet generator."""
    # Create sample data
    sample_data = TimesheetData(
        entries=[
            TimesheetEntry(month=11, day=1, start="09:00", end="17:00", paid_hours=8.0),
            TimesheetEntry(month=11, day=2, start="09:00", end="17:00", paid_hours=8.0),
            TimesheetEntry(month=11, day=3, start="09:00", end="13:00", paid_hours=4.0),
        ],
        total_hours=20.0,
    )

    create_sasi_timesheet(sample_data)

if __name__ == "__main__":
    main()
