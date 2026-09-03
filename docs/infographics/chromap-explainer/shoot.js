#!/usr/bin/env node
// Minimal CDP screenshot driver for the static infographic page.
// Usage: node shoot.js <file-url> <outdir>
// Produces:
//   render/slice-*.png      full-width fixed-height slices from y=0 (authoritative)
//   render/panels/<id>@2x.png  per-figure captures (visual inspection only)
//   render/page.json        { cssWidth, cssHeight, dpr } for the stitch assert
// The final full@2x.png is stitched from slices ONLY — per-section stitching
// loses inter-section margins and is forbidden here.
const { spawn } = require("child_process");
const fs = require("fs");
const path = require("path");

const SHELL = process.env.HEADLESS_SHELL || process.env.HOME +
  "/Library/Caches/ms-playwright/chromium_headless_shell-1234/chrome-headless-shell-mac-arm64/chrome-headless-shell";
const WIDTH = 1200;
const SLICE = 3600; // CSS px per slice (7200 device px, well under limits)

function connect(wsUrl) {
  return new Promise((res, rej) => {
    const ws = new WebSocket(wsUrl);
    ws.onerror = () => rej(new Error("ws error"));
    ws.onopen = () => res(ws);
  });
}

function cdpClient(ws) {
  let id = 0;
  const pending = new Map();
  const waiters = [];
  ws.onmessage = (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.id && pending.has(msg.id)) {
      const { res, rej } = pending.get(msg.id);
      pending.delete(msg.id);
      msg.error ? rej(new Error(msg.error.message)) : res(msg.result);
    } else if (msg.method) {
      for (let i = waiters.length - 1; i >= 0; i--) {
        if (waiters[i].method === msg.method &&
            (!waiters[i].sessionId || waiters[i].sessionId === msg.sessionId)) {
          const w = waiters.splice(i, 1)[0];
          w.res(msg.params);
        }
      }
    }
  };
  return {
    send(method, params = {}, sessionId) {
      const mid = ++id;
      return new Promise((res, rej) => {
        pending.set(mid, { res, rej });
        ws.send(JSON.stringify({ id: mid, method, params, sessionId }));
      });
    },
    once(method, sessionId) {
      return new Promise((res) => waiters.push({ method, res, sessionId }));
    },
  };
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

(async () => {
  const url = process.argv[2];
  const outdir = process.argv[3];
  fs.mkdirSync(path.join(outdir, "panels"), { recursive: true });
  const udd = fs.mkdtempSync("/tmp/shoot-profile-");
  const proc = spawn(SHELL, [
    "--remote-debugging-port=0",
    `--user-data-dir=${udd}`,
    "--no-first-run", "--no-default-browser-check",
    "--disable-gpu", "--hide-scrollbars", "--force-color-profile=srgb",
    "about:blank",
  ], { stdio: ["ignore", "ignore", "pipe"] });
  proc.stderr.on("data", () => {});
  const portFile = path.join(udd, "DevToolsActivePort");
  for (let i = 0; i < 100 && !fs.existsSync(portFile); i++) await sleep(100);
  const port = fs.readFileSync(portFile, "utf8").split("\n")[0];
  const ver = await (await fetch(`http://127.0.0.1:${port}/json/version`)).json();
  const ws = await connect(ver.webSocketDebuggerUrl);
  const c = cdpClient(ws);

  const { targetId } = await c.send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await c.send("Target.attachToTarget", { targetId, flatten: true });
  await c.send("Page.enable", {}, sessionId);
  await c.send("Runtime.enable", {}, sessionId);

  const loaded = c.once("Page.loadEventFired", sessionId);
  await c.send("Page.navigate", { url }, sessionId);
  await loaded;
  // fonts settled + two rAFs before any pixel is trusted
  await c.send("Runtime.evaluate", {
    expression: "document.fonts.ready.then(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))))",
    awaitPromise: true,
  }, sessionId);

  const dims = await c.send("Runtime.evaluate", {
    expression: "JSON.stringify({w: document.documentElement.scrollWidth, h: document.documentElement.scrollHeight})",
    returnByValue: true,
  }, sessionId);
  const { w, h } = JSON.parse(dims.result.value);
  console.log(`page ${w}x${h}`);
  if (w > WIDTH) {
    const over = await c.send("Runtime.evaluate", {
      expression: `JSON.stringify(Array.from(document.querySelectorAll('*'))
        .filter(e => e.getBoundingClientRect().right > ${WIDTH + 1})
        .slice(0, 24).map(e => (e.id || e.className || e.tagName) + ' right=' + Math.round(e.getBoundingClientRect().right)))`,
      returnByValue: true,
    }, sessionId);
    console.error("horizontal overflow:", over.result && over.result.value);
    process.exit(1);
  }
  fs.writeFileSync(path.join(outdir, "page.json"),
    JSON.stringify({ cssWidth: w, cssHeight: h, dpr: 2, slice: SLICE }, null, 1));

  const figs = await c.send("Runtime.evaluate", {
    expression: "JSON.stringify(Array.from(document.querySelectorAll('figure')).map(e => ({id: e.id, y: Math.round(e.getBoundingClientRect().top + window.scrollY), h: Math.round(e.getBoundingClientRect().height)})))",
    returnByValue: true,
  }, sessionId);
  if (!figs.result.value) {
    console.error("figs eval failed:", JSON.stringify(figs).slice(0, 400));
    process.exit(1);
  }
  fs.writeFileSync(path.join(outdir, "figs.json"), figs.result.value);

  // Deterministic capture: resize viewport to the region height, scroll the
  // region to the viewport origin (asserted), then screenshot the viewport.
  // (captureBeyondViewport+clip proved non-deterministic in this shell: one
  // run honored clip y, the next returned content from ~y+3200.)
  async function regionPNG(y, height) {
    await c.send("Emulation.setDeviceMetricsOverride", {
      width: WIDTH, height, deviceScaleFactor: 2, mobile: false,
    }, sessionId);
    const settled = await c.send("Runtime.evaluate", {
      expression: `window.scrollTo(0, ${y}); new Promise(r => requestAnimationFrame(() => requestAnimationFrame(() => r(JSON.stringify({sy: window.scrollY})))))`,
      awaitPromise: true, returnByValue: true,
    }, sessionId);
    const sy = JSON.parse(settled.result.value).sy;
    if (Math.abs(sy - y) > 1) {
      throw new Error(`scroll assert failed: want ${y} got ${sy}`);
    }
    const shot = await c.send("Page.captureScreenshot", {
      format: "png",
    }, sessionId);
    return Buffer.from(shot.data, "base64");
  }

  const slices = [];
  for (let y = 0; y < h; y += SLICE) {
    const hh = Math.min(SLICE, h - y);
    const buf = await regionPNG(y, hh);
    const p = path.join(outdir, `slice-${String(y).padStart(5, "0")}.png`);
    fs.writeFileSync(p, buf);
    slices.push(p);
    console.log("slice", y, buf.length, "bytes");
  }
  for (const f of JSON.parse(figs.result.value)) {
    const buf = await regionPNG(f.y, f.h);
    fs.writeFileSync(path.join(outdir, "panels", `${f.id}@2x.png`), buf);
    console.log("panel", f.id, f.y, f.h);
  }
  fs.writeFileSync(path.join(outdir, "slices.txt"), slices.join("\n") + "\n");
  proc.kill(9);
  process.exit(0);
})().catch((e) => { console.error(e); process.exit(1); });
