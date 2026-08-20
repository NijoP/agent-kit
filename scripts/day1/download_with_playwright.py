#!/usr/bin/env python3
"""
Download datasheets using Playwright browser automation.
"""

import asyncio
import os
import csv
import subprocess
from pathlib import Path

async def download_datasheet(page, mpn, manufacturer, url, output_path):
    """Try to download a datasheet using Playwright."""
    try:
        print(f"  Navigating to {url}")
        response = await page.goto(url, wait_until="networkidle", timeout=60000)
        
        if response and response.status == 200:
            content_type = response.headers.get("content-type", "")
            if "pdf" in content_type.lower():
                # Direct PDF download
                body = await response.body()
                if body.startswith(b"%PDF"):
                    with open(output_path, "wb") as f:
                        f.write(body)
                    return True, "direct_pdf"
            
            # Check if page has a download link
            download_links = await page.query_selector_all('a[href$=".pdf"], a[href*="datasheet"]')
            for link in download_links:
                href = await link.get_attribute("href")
                if href and ".pdf" in href:
                    # Try to download the PDF
                    pdf_response = await page.goto(href, wait_until="networkidle", timeout=30000)
                    if pdf_response and pdf_response.status == 200:
                        pdf_content_type = pdf_response.headers.get("content-type", "")
                        if "pdf" in pdf_content_type.lower():
                            body = await pdf_response.body()
                            if body.startswith(b"%PDF"):
                                with open(output_path, "wb") as f:
                                    f.write(body)
                                return True, f"link_pdf:{href}"
        
        return False, f"no_pdf_found (status: {response.status if response else 'none'})"
    except Exception as e:
        return False, f"error: {str(e)}"

async def test_yageo(page, mpn):
    """Test Yageo datasheet download."""
    urls = [
        f"https://www.yageo.com/en/Products/Detail/{mpn}",
        f"https://www.yageo.com/upload/media/product/{mpn}.pdf",
        f"https://www.yageogroup.com/upload/media/product/{mpn}.pdf",
    ]
    
    for url in urls:
        success, msg = await download_datasheet(page, mpn, "Yageo", url, f"/tmp/{mpn}.pdf")
        if success:
            return True, msg, url
    return False, "all_failed", ""

async def test_vishay(page, mpn):
    """Test Vishay datasheet download."""
    urls = [
        f"https://www.vishay.com/docs/28746/{mpn}.pdf",
        f"https://www.vishay.com/doc?{mpn}",
    ]
    
    for url in urls:
        success, msg = await download_datasheet(page, mpn, "Vishay", url, f"/tmp/{mpn}.pdf")
        if success:
            return True, msg, url
    return False, "all_failed", ""

async def main():
    """Test Playwright downloads for a few manufacturers."""
    from playwright.async_api import async_playwright
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page()
        page.set_default_timeout(60000)
        
        # Test Yageo RC0402FR-071RL
        print("Testing Yageo RC0402FR-071RL...")
        success, msg, url = await test_yageo(page, "RC0402FR-071RL")
        print(f"  Result: {success} - {msg} ({url})")
        
        if success and os.path.exists("/tmp/RC0402FR-071RL.pdf"):
            result = subprocess.run(['file', '/tmp/RC0402FR-071RL.pdf'], capture_output=True, text=True)
            print(f"  File: {result.stdout.strip()}")
        
        # Test Vishay CRCW04021R00FKED
        print("\nTesting Vishay CRCW04021R00FKED...")
        success, msg, url = await test_vishay(page, "CRCW04021R00FKED")
        print(f"  Result: {success} - {msg} ({url})")
        
        if success and os.path.exists("/tmp/CRCW04021R00FKED.pdf"):
            result = subprocess.run(['file', '/tmp/CRCW04021R00FKED.pdf'], capture_output=True, text=True)
            print(f"  File: {result.stdout.strip()}")
        
        await browser.close()

if __name__ == "__main__":
    asyncio.run(main())