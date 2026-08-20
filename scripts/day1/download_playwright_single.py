#!/usr/bin/env python3
"""
Use Playwright to find and download datasheets from manufacturer/distributor sites.
"""

import asyncio
import os
import subprocess
from playwright.async_api import async_playwright

async def find_and_download_datasheet(page, mpn, manufacturer, output_path):
    """Try multiple sources to find a datasheet."""
    
    sources = [
        # Distributor sources (often more accessible)
        f"https://www.digikey.com/en/products/detail/{manufacturer.lower()}/{mpn}",
        f"https://www.mouser.com/ProductDetail/{manufacturer}/{mpn}",
        f"https://www.farnell.com/search?q={mpn}",
        f"https://www.arrow.com/en/products/search?q={mpn}",
        f"https://www.rs-online.com/search?q={mpn}",
        
        # Manufacturer sources
        f"https://www.{manufacturer.lower().replace(' ', '')}.com/en/Products/Detail/{mpn}",
    ]
    
    for url in sources:
        try:
            print(f"  Trying: {url}")
            await page.goto(url, wait_until="domcontentloaded", timeout=30000)
            await page.wait_for_timeout(2000)
            
            # Look for PDF links
            pdf_links = await page.query_selector_all('a[href$=".pdf"], a[href*="datasheet"]')
            
            for link in pdf_links:
                href = await link.get_attribute("href")
                if href and ".pdf" in href:
                    # Make absolute URL
                    if href.startswith("/"):
                        from urllib.parse import urlparse
                        parsed = urlparse(url)
                        href = f"{parsed.scheme}://{parsed.netloc}{href}"
                    elif not href.startswith("http"):
                        continue
                    
                    print(f"    Found PDF link: {href}")
                    
                    # Try to download
                    try:
                        response = await page.goto(href, wait_until="networkidle", timeout=30000)
                        if response and response.status == 200:
                            content_type = response.headers.get("content-type", "")
                            if "pdf" in content_type.lower():
                                body = await response.body()
                                if body.startswith(b"%PDF"):
                                    with open(output_path, "wb") as f:
                                        f.write(body)
                                    return True, href
                    except Exception as e:
                        print(f"    Download failed: {e}")
                        continue
        except Exception as e:
            print(f"    Error with {url}: {e}")
            continue
    
    return False, None

async def test_yageo():
    """Test downloading Yageo RC0402FR-071RL datasheet."""
    mpn = "RC0402FR-071RL"
    manufacturer = "Yageo"
    output_path = "/tmp/yageo_test.pdf"
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page()
        page.set_default_timeout(60000)
        
        print(f"Testing {manufacturer} {mpn}...")
        
        # Try DigiKey first
        url = f"https://www.digikey.com/en/products/detail/yageo/{mpn}"
        print(f"Trying DigiKey: {url}")
        
        try:
            await page.goto(url, wait_until="domcontentloaded", timeout=60000)
            await page.wait_for_timeout(3000)
            
            # Look for datasheet link
            pdf_links = await page.query_selector_all('a[href$=".pdf"]')
            print(f"Found {len(pdf_links)} PDF links on DigiKey")
            
            for link in pdf_links:
                href = await link.get_attribute("href")
                print(f"  Link: {href}")
                if href and ".pdf" in href:
                    if href.startswith("/"):
                        href = f"https://www.digikey.com{href}"
                    try:
                        response = await page.goto(href, wait_until="networkidle", timeout=30000)
                        if response and response.status == 200:
                            content_type = response.headers.get("content-type", "")
                            if "pdf" in content_type.lower():
                                body = await response.body()
                                if body.startswith(b"%PDF"):
                                    with open("/tmp/yageo_digikey.pdf", "wb") as f:
                                        f.write(body)
                                    print(f"  SUCCESS: Downloaded from {href}")
                                    return True
                    except Exception as e:
                        print(f"    Error: {e}")
        except Exception as e:
            print(f"DigiKey error: {e}")
        
        # Try Mouser
        url = f"https://www.mouser.com/ProductDetail/Yageo/{mpn}"
        print(f"Trying Mouser: {url}")
        
        try:
            await page.goto(url, wait_until="domcontentloaded", timeout=60000)
            await page.wait_for_timeout(3000)
            
            pdf_links = await page.query_selector_all('a[href$=".pdf"]')
            print(f"Found {len(pdf_links)} PDF links on Mouser")
            
            for link in pdf_links:
                href = await link.get_attribute("href")
                print(f"  Link: {href}")
                if href and ".pdf" in href:
                    if href.startswith("/"):
                        href = f"https://www.mouser.com{href}"
                    try:
                        response = await page.goto(href, wait_until="networkidle", timeout=30000)
                        if response and response.status == 200:
                            content_type = response.headers.get("content-type", "")
                            if "pdf" in content_type.lower():
                                body = await response.body()
                                if body.startswith(b"%PDF"):
                                    with open("/tmp/yageo_mouser.pdf", "wb") as f:
                                        f.write(body)
                                    print(f"  SUCCESS: Downloaded from {href}")
                                    return True
                    except Exception as e:
                        print(f"    Error: {e}")
        except Exception as e:
            print(f"Mouser error: {e}")
        
        await browser.close()
        return False

if __name__ == "__main__":
    asyncio.run(test_yageo())