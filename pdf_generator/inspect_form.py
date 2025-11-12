"""
Utility script to inspect PDF form fields.
Run this to see what fields are available in your template.
"""

import json
from PyPDFForm import PdfWrapper


def inspect_pdf_form(template_path: str = "../templates/sasi.pdf"):
    """
    Inspect the form fields in a PDF template.

    Args:
        template_path: Path to the PDF template file
    """
    try:
        pdf = PdfWrapper(template_path, adobe_mode=True)

        schema = pdf.schema["properties"]

        print("PDF Form Fields Found:")
        print("=" * 60)

        if isinstance(schema, dict):
            for idx, field_name in enumerate(schema.keys(), 1):
                print(f"{idx:3}. {field_name}")
        else:
            print(schema)

        print("=" * 60)
        print(f"Total fields: {len(schema) if isinstance(schema, dict) else 'unknown'}")

        # print(json.dumps(schema, indent=4, sort_keys=True))

    except FileNotFoundError:
        print(f"Error: Template file not found at {template_path}")
        print("Please ensure the template exists at the specified path.")
    except Exception as e:
        print(f"Error inspecting PDF: {e}")


if __name__ == "__main__":
    inspect_pdf_form()
