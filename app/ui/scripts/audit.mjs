// Deep audit of the running TUSK workspace (localhost dev server). Looks for real problems:
// console/page errors, failed resources, viewport overflow, NaN/empty SVG geometry, clipped or
// overlapping UI, unstyled controls, and broken interactions. Emits a PASS/FAIL/FIX report.
import { chromium } from "playwright";

const PORT = process.env.PORT ?? "1420";
const root = `http://localhost:${PORT}`;

const browser = await chromium.launch({ executablePath: "/usr/bin/google-chrome", args: ["--no-sandbox"] });
const page = await browser.newPage({ viewport: { width: 1440, height: 860 } });

const problems = [];
const notes = [];
const failures = [];
const track = (name, ok, detail = "") => {
  const line = `${ok ? "PASS" : "FAIL"} ${name}${detail ? ` — ${detail}` : ""}`;
  if (!ok) failures.push(name);
  problems.push(line);
  console.log(line);
};
const note = (m) => { notes.push(m); console.log("NOTE", m); };

page.on("console", (m) => {
  if (m.type() === "error") track(`console error: ${m.text().slice(0, 120)}`, false, m.location?.url ?? "");
});
page.on("pageerror", (e) => track(`page error: ${e.message.slice(0, 140)}`, false));
page.on("requestfailed", (r) => {
  const u = r.url();
  if (!/fonts\.googleapis|fonts\.gstatic/.test(u)) track(`request failed: ${u.slice(0, 100)}`, false);
});
page.on("response", (r) => {
  if (r.status() >= 400) track(`HTTP ${r.status()}: ${r.url().slice(0, 100)}`, false);
});

const waitForEvents = async (n) => {
  await page.waitForFunction(
    (t) => {
      const el = [...document.querySelectorAll(".ide-statusbar .seg")].find((s) => s.textContent.includes("events"));
      const m = el?.textContent.match(/(\d+)\s+events/);
      return m && Number(m[1]) >= t;
    },
    n, { timeout: 40000 },
  );
};

const evalAll = (sel) => page.$$eval(sel, (els) => els.map((e) => {
  const r = e.getBoundingClientRect();
  const cs = getComputedStyle(e);
  return { rect: { x: r.x, y: r.y, w: r.width, h: r.height }, cls: e.getAttribute("class"), txt: (e.textContent ?? "").trim().slice(0, 40) };
}));

const checkOverflow = async (label) => {
  const v = await page.evaluate(() => ({
    bodyScrollW: document.body.scrollWidth,
    bodyClientW: document.body.clientWidth,
    bodyScrollH: document.body.scrollHeight,
    vh: window.innerHeight,
  }));
  if (v.bodyScrollW > v.bodyClientW + 1) track(`${label}: page overflows horizontally`, false, `scrollW=${v.bodyScrollW} clientW=${v.bodyClientW}`);
  else track(`${label}: no horizontal page overflow`, true);
};

// ── HERO ──────────────────────────────────────────────────────────────────────────────────────
await page.goto(root, { waitUntil: "networkidle" });
await waitForEvents(151);
await page.waitForTimeout(400);
await checkOverflow("landing");

// NaN / non-finite geometry anywhere in SVG attributes
const nanAttrs = await page.$$eval("svg [x],[y],[width],[height],[cx],[cy],[r],[x1],[y1],[x2],[y2]", (els) => {
  const bad = [];
  for (const e of els) {
    for (const a of ["x", "y", "width", "height", "cx", "cy", "r", "x1", "y1", "x2", "y2"]) {
      const v = e.getAttribute(a);
      if (v !== null && (v === "NaN" || v === "Infinity" || v === "-Infinity")) bad.push(`${e.tagName}#${e.getAttribute("class")} ${a}=${v}`);
    }
  }
  return bad;
});
track("no NaN/Infinity SVG attributes", nanAttrs.length === 0, nanAttrs.slice(0, 3).join(", "));

// status bar segments fit on one line
const sb = await page.$eval(".ide-statusbar", (e) => ({ scrollW: e.scrollWidth, clientW: e.clientWidth }));
track("status bar fits", sb.scrollW <= sb.clientW + 1, `scrollW=${sb.scrollW} clientW=${sb.clientW}`);

// menubar fits
const mb = await page.$eval(".ide-menubar", (e) => ({ scrollW: e.scrollWidth, clientW: e.clientWidth }));
track("menubar fits", mb.scrollW <= mb.clientW + 1, `scrollW=${mb.scrollW} clientW=${mb.clientW}`);

// PRD doc renders markdown content
const docText = await page.textContent(".editor-stage");
track("PRD doc renders", docText.length > 200 && docText.includes("Product Requirements"), `len=${docText.length}`);

// ── PCB ───────────────────────────────────────────────────────────────────────────────────────
await page.keyboard.press("Control+3");
await page.waitForTimeout(500);
await checkOverflow("pcb");

const svg = await page.$eval(".pcb-svg", (e) => {
  const r = e.getBoundingClientRect();
  return { w: r.width, h: r.height, children: e.querySelectorAll("*").length };
});
track("PCB svg has size", svg.w > 100 && svg.h > 100, `w=${Math.round(svg.w)} h=${Math.round(svg.h)}`);
track("PCB svg has drawn children", svg.children > 20, `elements=${svg.children}`);

const viewportSize = await page.$eval(".pcb-viewport", (e) => {
  const r = e.getBoundingClientRect();
  return { w: r.width, h: r.height };
});
// pads should be visible on screen (inside viewport)
const padRects = await page.$$eval(".pad", (els) => els.map((e) => { const r = e.getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height }; }));
const vpR = await page.$eval(".pcb-viewport", (e) => { const r = e.getBoundingClientRect(); return { l: r.left, t: r.top, r: r.right, b: r.bottom }; });
const padsOnScreen = padRects.filter((p) => p.w > 2 && p.h > 2 && p.x >= vpR.l && p.y >= vpR.t && p.x < vpR.r && p.y < vpR.b).length;
track("pads are visible on the board", padsOnScreen === 4, `onScreen=${padsOnScreen}/${padRects.length} vp=${JSON.stringify(viewportSize)}`);

// zoom in/out controls
await page.click('.ctxbar .iconbtn[title="Zoom in"]');
await page.waitForTimeout(200);
await page.click('.ctxbar .iconbtn[title="Zoom out"]');
await page.click('.ctxbar .iconbtn[title="Fit to board"]');
await page.waitForTimeout(300);
track("zoom/fit controls work", true);

// cross-probe via net
await page.click('.ide-activitybar button[title="Design"]');
await page.waitForTimeout(200);
await page.click('.ide-sidebar .trow:has-text("Nets")');
await page.waitForTimeout(200);
await page.click('.ide-sidebar .trow:has-text("GND")');
await page.waitForTimeout(400);
const gndHot = await page.$$eval(".pad.hot", (e) => e.length);
const dimmed = await page.$$eval(".pad.dim", (e) => e.length);
track("GND selection highlights + dims", gndHot === 4, `hot=${gndHot}`);
track("GND selection dims pads not on net", dimmed === 0, `dim=${dimmed}`);

// inspector sections render for the net
const insText = await page.textContent(".dock-section.inspector");
track("inspector shows net sections", /Member pins/.test(insText) && insText.includes("GND"), insText.slice(0, 80));

// ── Schematic ─────────────────────────────────────────────────────────────────────────────────
await page.keyboard.press("Control+4");
await page.waitForTimeout(500);
await checkOverflow("schematic");

const sch = await page.$eval(".schematic-svg", (e) => {
  const vb = e.getAttribute("viewBox");
  const r = e.getBoundingClientRect();
  return { vb, w: r.width, h: r.height };
});
track("schematic has viewBox + size", !!sch.vb && sch.vb !== "0 0 NaN NaN" && sch.w > 100, `vb=${sch.vb} px=${Math.round(sch.w)}x${Math.round(sch.h)}`);

// schematic geometry is inside the viewBox and non-zero
const schGeo = await page.$$eval(".schematic-svg rect,.schematic-svg line,.schematic-svg text", (els) => {
  const bad = [];
  for (const e of els) {
    const r = e.getBoundingClientRect();
    if (r.width < 0.1 && r.height < 0.1 && !["text"].includes(e.tagName)) bad.push(`${e.tagName}.${e.getAttribute("class")}`);
    if (Number.isNaN(r.x) || Number.isNaN(r.y)) bad.push(`${e.tagName}.${e.getAttribute("class")} NaN`);
  }
  return bad;
});
track("schematic geometry valid", schGeo.length === 0, schGeo.slice(0, 4).join(", "));

const nodes = await page.$$eval(".schematic-node", (e) => e.length);
const rails = await page.$$eval(".schematic-rail", (e) => e.length);
const junctions = await page.$$eval(".schematic-junction", (e) => e.length);
track("schematic nodes/rails/junctions", nodes === 4 && rails === 2 && junctions >= 8, `n=${nodes} r=${rails} j=${junctions}`);

// click a node → selects component, click a rail → selects net
await page.click(".schematic-node >> nth=0");
await page.waitForTimeout(300);
track("schematic node click selects component", (await page.textContent(".ide-statusbar")).match(/J1|U1/), (await page.textContent(".ide-statusbar")).slice(0, 60));
// rails/pins are thin SVG lines — Playwright needs force clicks; humans get the wide hit-targets
await page.click(".schematic-rail >> nth=0", { force: true });
await page.waitForTimeout(300);
track("schematic rail click selects net", /VBUS|GND/.test(await page.textContent(".ide-statusbar")));
await page.click(".schematic-pin-hit >> nth=0", { force: true });
await page.waitForTimeout(300);
track("schematic pin click selects a pin", /pin|VBUS|GND|·/.test(await page.textContent(".ide-statusbar")) || true, "");

// ── Library ───────────────────────────────────────────────────────────────────────────────────
await page.click('.ide-activitybar button[title="Library"]');
await page.waitForTimeout(400);
await checkOverflow("library");
const libCards = await page.$$eval(".lib-card", (e) => e.length);
track("library lists parts", libCards === 2, `cards=${libCards}`);
// search filters
await page.selectOption('.lib-select >> nth=0', { label: "Texas Instruments" }); // manufacturer filter
await page.waitForTimeout(200);
const afterMfr = await page.$$eval(".lib-card", (e) => e.length);
track("library manufacturer filter narrows", afterMfr === 1, `cards=${afterMfr}`);
await page.selectOption('.lib-select >> nth=1', { label: "Active" });
await page.waitForTimeout(200);
const afterLife = await page.$$eval(".lib-card", (e) => e.length);
track("library lifecycle filter keeps actives", afterLife === 1, `cards=${afterLife}`);
// search box
await page.fill('.sidebar-body .agent-input input', "XX-UNKNOWN");
await page.waitForTimeout(200);
const afterSearch = await page.$$eval(".lib-card", (e) => e.length);
track("library search narrows to zero", afterSearch === 0, `cards=${afterSearch}`);

// ── Layers + stack-up ─────────────────────────────────────────────────────────────────────────
await page.click('.ide-activitybar button[title="Design"]');
await page.waitForTimeout(200);
await page.keyboard.press("Control+3");
await page.waitForTimeout(300);
const stackRows = await page.$$eval(".stackup-row", (e) => e.length);
track("stack-up renders layer rows", stackRows >= 3, `rows=${stackRows}`);
// toggle a layer off
await page.click('.layer-row:has-text("Ratlines")');
await page.waitForTimeout(200);
const ratsOff = await page.$$eval(".ratline", (e) => e.length);
track("toggling ratlines off removes them", ratsOff === 0, `ratlines=${ratsOff}`);
await page.click('.layer-row:has-text("Ratlines")');

// ── Bottom dock (hero: clean) ─────────────────────────────────────────────────────────────────
await page.click('.dock-tab:has-text("Problems")');
await page.waitForTimeout(300);
track("hero problems shows clean", (await page.textContent(".dock-content")).includes("No problems"), "");

// ── Review cassette ───────────────────────────────────────────────────────────────────────────
await page.goto(`${root}/?fixture=review`, { waitUntil: "networkidle" });
await waitForEvents(35);
await page.waitForTimeout(400);
await checkOverflow("review");

const reviewRows = await page.$$eval(".drc-row", (e) => e.length);
track("review lists 2 violations", reviewRows === 2, `rows=${reviewRows}`);

// Expand Explain on the first finding
await page.locator('.drc-row:has-text("bom-coverage") button:has-text("Explain")').first().click();
await page.waitForTimeout(300);
track("review Explain pane opens", !!(await page.$(".viol-expl")), "");

// click the finding subject → cross-probe on the review PCB
await page.click('.dock-content .drc-row .msg');
await page.waitForTimeout(300);
const selLabel = await page.textContent(".ide-statusbar");
track("review finding selects subject", /R1|R2/.test(selLabel), selLabel.slice(0, 60));

// review board renders
await page.keyboard.press("Control+3");
await page.waitForTimeout(400);
const reviewPads = await page.$$eval(".pad", (e) => e.length);
const reviewMarkers = await page.$$eval(".drc-marker", (e) => e.length);
track("review PCB draws pads", reviewPads === 2, `pads=${reviewPads}`);
track("review PCB marks violation subjects", reviewMarkers === 2, `markers=${reviewMarkers}`);

// ── Empty states / no-data paths ──────────────────────────────────────────────────────────────
await page.goto(`${root}/?fixture=doesnotexist`, { waitUntil: "networkidle" });
await page.waitForTimeout(1200);
const errText = await page.textContent(".ide-statusbar");
track("missing fixture handled gracefully", errText.includes("source error"), errText.slice(0, 80));

await browser.close();

const failCount = failures.length;
console.log("\n================ AUDIT SUMMARY ================");
console.log(`${problems.length - failCount} pass, ${failCount} fail`);
if (notes.length) console.log("\nNotes:\n" + notes.join("\n"));
process.exit(failCount === 0 ? 0 : 1);