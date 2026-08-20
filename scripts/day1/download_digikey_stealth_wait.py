#!/usr/bin/env python3
"""
Use Playwright with stealth and proper waiting to download datasheets from DigiKey.
"""

import asyncio
import os
import subprocess
from playwright.async_api import async_playwright
from playwright_stealth import Stealth

async def download_from_digikey(page, mpn, output_path):
    """Try to download datasheet from DigiKey using stealth with proper waiting."""
    url = f"https://www.digikey.com/en/products/detail/yageo/{mpn}"
    print(f"  Navigating to: {url}")
    
    try:
        stealth = Stealth()
        await stealth.apply_stealth_async(page)
        
        response = await page.goto(url, wait_until="networkidle", timeout=90000)
        print(f"  Page loaded, status: {response.status if response else 'unknown'}")
        
        # Wait for the page to fully render - wait for common elements
        await page.wait_for_timeout(10000)
        
        # Try to find the datasheet link by waiting for it to appear
        print("  Waiting for datasheet link to appear...")
        try:
            # Wait for any PDF link to appear
            await page.wait_for_selector('a[href$=".pdf"]', timeout=30000)
            print("  Found PDF link!")
        except:
            print("  No PDF link found after waiting")
        
        # Try multiple selectors after waiting
        selectors = [
            'a[href*="datasheet"]',
            'a[href$=".pdf"]',
            '[data-testid="datasheet-link"]',
            '.datasheet-link',
            'a[href*="htmldatasheets"]',
            'a[href*="datasheet"]',
        ]
        
        for selector in selectors:
            links = await page.query_selector_all(selector)
            print(f"  Selector '{selector}': found {len(links)} links")
            
            for link in links:
                href = await link.get_attribute("href")
                if href and (".pdf" in href or "datasheet" in href.lower()):
                    if href.startswith("/"):
                        href = f"https://www.digikey.com{href}"
                    elif not href.startswith("http"):
                        continue
                    
                    print(f"    Found datasheet link: {href}")
                    try:
                        response = await page.goto(href, wait_until="networkidle", timeout=30000)
                        if response and response.status == 200:
                            content_type = response.headers.get("content-type", "")
                            if "pdf" in content_type.lower():
                                body = await response.body()
                                if body.startswith(b"%PDF"):
                                    with open(output_path, "wb") as f:
                                        f.write(body)
                                    print(f"    SUCCESS: Downloaded from {href}")
                                    return True, href
                                else:
                                    print(f"    Not a PDF: {content_type}")
                            else:
                                print(f"    HTTP {response.status}: {content_type}")
                    except Exception as e:
                        print(f"    Download error: {e}")
                        continue
        
        return False, None
    except Exception as e:
        print(f"  Error: {e}")
        return False, None

async def test_digikey_stealth():
    mpn = "RC0402FR-071RL"
    output_path = "/tmp/digikey_stealth.pdf"
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page()
        page.set_default_timeout(120000)
        
        print(f"Testing DigiKey with stealth for {mpn}...")
        success, url = await download_from_digikey(page, mpn, output_path)
        print(f"Result: {success} - {url}")
        
        if success and os.path.exists(output_path):
            result = subprocess.run(['file', output_path], capture_output=True, text=True)
            print(f"File: {result.stdout.strip()}")
        
        await browser.close()
        return success

if __name__ == "__main__":
    import os
    asyncio.run(test_digikey_stealth())