#!/usr/bin/env python3
"""
Use Playwright to search componentsearchengine.com and download datasheets.
"""

import asyncio
import os
import subprocess
from playwright.async_api import async_playwright

async def search_and_download(page, mpn, output_path):
    """Search for MPN on componentsearchengine.com and try to download datasheet."""
    try:
        # Search for the part
        search_url = f"https://componentsearchengine.com/search?q={mpn}"
        print(f"  Searching: {search_url}")
        await page.goto(search_url, wait_until="domcontentloaded", timeout=60000)
        await page.wait_for_timeout(3000)
        
        # Look for part links
        part_links = await page.query_selector_all('a[href*="/parts/"]')
        print(f"  Found {len(part_links)} part links")
        
        for link in part_links[:3]:  # Try first 3 results
            href = await link.get_attribute("href")
            if href:
                if href.startswith("/"):
                    href = f"https://componentsearchengine.com{href}"
                print(f"  Checking part page: {href}")
                try:
                    await page.goto(href, wait_until="domcontentloaded", timeout=30000)
                    await page.wait_for_timeout(2000)
                    
                    # Look for datasheet download link
                    datasheet_links = await page.query_selector_all('a[href$=".pdf"], a[href*="datasheet"]')
                    for dl in datasheet_links:
                        dl_href = await dl.get_attribute("href")
                        if dl_href and ".pdf" in dl_href:
                            if dl_href.startswith("/"):
                                dl_href = f"https://componentsearchengine.com{dl_href}"
                            print(f"    Found datasheet link: {dl_href}")
                            try:
                                response = await page.goto(dl_href, wait_until="networkidle", timeout=30000)
                                if response and response.status == 200:
                                    content_type = response.headers.get("content-type", "")
                                    if "pdf" in content_type.lower():
                                        body = await response.body()
                                        if body.startswith(b"%PDF"):
                                            with open(output_path, "wb") as f:
                                                f.write(body)
                                            print(f"    SUCCESS: Downloaded from {dl_href}")
                                            return True, dl_href
                            except Exception as e:
                                print(f"    Download error: {e}")
                except Exception as e:
                    print(f"    Part page error: {e}")
                    continue
    except Exception as e:
        print(f"Search error: {e}")
    
    return False, None

async def test_componentsearchengine():
    mpn = "RC0402FR-071RL"
    output_path = "/tmp/cse_test.pdf"
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page()
        page.set_default_timeout(60000)
        
        print(f"Testing componentsearchengine.com for {mpn}...")
        success, url = await search_and_download(page, mpn, output_path)
        print(f"Result: {success} - {url}")
        
        if success and os.path.exists(output_path):
            result = subprocess.run(['file', output_path], capture_output=True, text=True)
            print(f"File: {result.stdout.strip()}")
        
        await browser.close()
        return success

if __name__ == "__main__":
    import os
    asyncio.run(test_componentsearchengine())