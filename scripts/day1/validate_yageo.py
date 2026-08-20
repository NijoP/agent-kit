#!/usr/bin/env python3
"""
Validate RC0402FR-071RL against authoritative Yageo documentation.
"""

import asyncio
import json
from playwright.async_api import async_playwright

async def extract_yageo_specs(page, mpn):
    """Extract specs from Yageo product page."""
    url = f"https://www.yageo.com/en/Products/Detail/{mpn}"
    print(f"Loading: {url}")
    
    await page.goto(url, wait_until="domcontentloaded", timeout=90000)
    await page.wait_for_timeout(10000)
    
    # Try to extract specs from the page
    specs = {}
    
    # Try to find spec table
    spec_elements = await page.query_selector_all('table tr, .spec-row, .specification-row, [class*="spec"]')
    for elem in spec_elements:
        text = await elem.inner_text()
        text = text.strip().lower()
        if any(keyword in text for keyword in ['resistance', 'tolerance', 'power', 'tcr', 'ppm', 'voltage', 'package', 'size']):
            print(f"  Spec row: {text[:200]}")
    
    # Try to find structured data
    json_elements = await page.query_selector_all('script[type="application/json"], script[type="application/ld+json"]')
    for elem in json_elements:
        text = await elem.inner_text()
        try:
            data = json.loads(text)
            print(f"  Found JSON data: {json.dumps(data)[:500]}")
        except:
            pass
    
    # Try to get all text content and search for specs
    body_text = await page.inner_text('body')
    lines = body_text.split('\n')
    for line in lines:
        line_lower = line.lower().strip()
        if any(kw in line_lower for kw in ['resistance', 'tolerance', 'power', 'tcr', 'ppm', 'voltage', 'package', '0402', '1005']):
            if len(line) > 5 and len(line) < 300:
                print(f"  Found spec line: {line.strip()}")
    
    return specs

async def main():
    mpn = "RC0402FR-071RL"
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page()
        page.set_default_timeout(90000)
        
        print(f"Validating {mpn} against Yageo website...")
        await extract_yageo_specs(page, mpn)
        
        await browser.close()

if __name__ == "__main__":
    asyncio.run(main())