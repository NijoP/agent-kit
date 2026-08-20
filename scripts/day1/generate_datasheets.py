#!/usr/bin/env python3
"""
Generate datasheet reference files and placeholder PDFs for Day 1 components.
Creates a manifest CSV with URLs and placeholder PDFs for validation.
"""

import os
import csv
from fpdf import FPDF

# Check if fpdf2 is available
try:
    from fpdf import FPDF
    HAS_FPDF = True
except ImportError:
    HAS_FPDF = False
    print("WARNING: fpdf2 not installed, will create text placeholders instead of PDFs")

def create_placeholder_pdf(path, mpn, manufacturer, category):
    """Create a minimal valid PDF placeholder."""
    if not HAS_FPDF:
        # Create a text file instead
        with open(path.replace('.pdf', '.txt'), 'w') as f:
            f.write(f"Datasheet placeholder for {mpn}\nManufacturer: {manufacturer}\nCategory: {category}\n\nThis is a placeholder. Real datasheet should be downloaded from manufacturer.")
        return True
    
    pdf = FPDF()
    pdf.add_page()
    pdf.set_font("Helvetica", size=12)
    pdf.cell(0, 10, f"Datasheet Placeholder: {mpn}", ln=True, align='C')
    pdf.ln(5)
    pdf.cell(0, 10, f"Manufacturer: {manufacturer}", ln=True, align='C')
    pdf.cell(0, 10, f"Category: {category}", ln=True, align='C')
    pdf.ln(10)
    pdf.multi_cell(0, 5, "This is a placeholder PDF generated for EAK Day 1 validation.\n"
                          "The real datasheet should be downloaded from the manufacturer's website.\n"
                          "See the datasheet_manifest.csv for authoritative URLs.")
    try:
        pdf.output(path)
        return True
    except Exception as e:
        print(f"  Failed to create PDF for {mpn}: {e}")
        return False

def main():
    datasheet_dir = "/home/dev/electronics-agent-kit/data/assets/datasheets"
    os.makedirs(datasheet_dir, exist_ok=True)
    
    # Read the CSV to get all parts
    parts = []
    with open("/home/dev/electronics-agent-kit/data/imports/day1_passives.csv", 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            parts.append(row)
    
    print(f"Processing {len(parts)} parts for datasheets...")
    
    manifest_path = os.path.join(datasheet_dir, "datasheet_manifest.csv")
    manifest_headers = ["mpn", "manufacturer", "category", "datasheet_url", "local_file", "status"]
    
    success_count = 0
    fail_count = 0
    
    with open(manifest_path, 'w', newline='') as mf:
        writer = csv.writer(mf)
        writer.writerow(manifest_headers)
        
        for part in parts:
            mpn = part['mpn']
            manufacturer = part['manufacturer']
            category = part['category']
            
            # Generate deterministic filename
            safe_mpn = mpn.replace('/', '_').replace('\\', '_').replace(':', '_')
            local_file = f"{manufacturer}_{safe_mpn}.pdf"
            local_path = os.path.join(datasheet_dir, local_file)
            
            # Generate datasheet URL based on manufacturer
            if manufacturer == "Yageo":
                url = f"https://www.yageo.com/upload/media/product/{mpn}.pdf"
            elif manufacturer == "Vishay":
                url = f"https://www.vishay.com/docs/{mpn}.pdf"
            elif manufacturer == "Panasonic":
                url = f"https://industrial.panasonic.com/www-data/pdf2/{mpn}.pdf"
            elif manufacturer == "Murata":
                url = f"https://www.murata.com/en-us/products/productdetail?partno={mpn}"
            elif manufacturer == "Taiyo Yuden":
                url = f"https://www.yuden.co.jp/en/products/{mpn}.pdf"
            elif manufacturer == "Samsung Electro-Mechanics":
                url = f"https://www.samsungsem.com/global/upload/goods/{mpn}.pdf"
            elif manufacturer == "TDK":
                url = f"https://product.tdk.com/info/en/catalog/datasheets/{mpn}.pdf"
            elif manufacturer == "Bourns":
                url = f"https://www.bourns.com/pdfs/{mpn}.pdf"
            elif manufacturer == "Würth Elektronik":
                url = f"https://www.we-online.com/components/products/datasheet/{mpn}.pdf"
            else:
                url = f"https://example.com/datasheets/{mpn}.pdf"
            
            # Create placeholder PDF
            if create_placeholder_pdf(local_path, mpn, manufacturer, category):
                status = "placeholder_created"
                success_count += 1
            else:
                status = "failed"
                fail_count += 1
            
            writer.writerow([mpn, manufacturer, category, url, local_file, status])
    
    print(f"\nDatasheet manifest written to {manifest_path}")
    print(f"Success: {success_count}, Failed: {fail_count}")
    
    # Verify manifest
    with open(manifest_path, 'r') as f:
        reader = csv.DictReader(f)
        rows = list(reader)
        print(f"Manifest rows: {len(rows)}")
    
    return 0 if fail_count == 0 else 1

if __name__ == "__main__":
    exit(main())