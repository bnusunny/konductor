#!/usr/bin/env node
"use strict";

const https = require("https");
const http = require("http");
const fs = require("fs");
const path = require("path");
const crypto = require("crypto");

const REPO = "bnusunny/konductor";
const PLATFORM_MAP = {
  "darwin-x64": "konductor-macos-x86_64",
  "darwin-arm64": "konductor-macos-arm64",
  "linux-x64": "konductor-linux-x86_64",
  "linux-arm64": "konductor-linux-arm64",
};

function getAssetName() {
  const key = `${process.platform}-${process.arch}`;
  const name = PLATFORM_MAP[key];
  if (!name) {
    throw new Error(`Unsupported platform: ${key}. Supported: ${Object.keys(PLATFORM_MAP).join(", ")}`);
  }
  return name;
}

function fetch(url) {
  return new Promise((resolve, reject) => {
    const mod = url.startsWith("https") ? https : http;
    mod.get(url, { headers: { "User-Agent": "konductor-npm" } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        res.resume();
        return fetch(res.headers.location).then(resolve, reject);
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
      }
      const chunks = [];
      res.on("data", (c) => chunks.push(c));
      res.on("end", () => resolve(Buffer.concat(chunks)));
      res.on("error", reject);
    }).on("error", reject);
  });
}

async function main() {
  const pkg = JSON.parse(fs.readFileSync(path.join(__dirname, "..", "package.json"), "utf8"));
  const version = pkg.version;
  const asset = getAssetName();
  const baseUrl = `https://github.com/${REPO}/releases/download/v${version}`;
  const binDir = path.join(__dirname, "..", "bin");
  const dest = path.join(binDir, "konductor");

  console.log(`Downloading konductor v${version} (${asset})...`);

  const binary = await fetch(`${baseUrl}/${asset}`);

  // Verify checksum
  try {
    const checksumData = await fetch(`${baseUrl}/${asset}.sha256`);
    const expected = checksumData.toString("utf8").trim().split(/\s+/)[0];
    const actual = crypto.createHash("sha256").update(binary).digest("hex");
    if (expected !== actual) {
      throw new Error(`Checksum mismatch: expected ${expected}, got ${actual}`);
    }
    console.log("Checksum verified.");
  } catch (e) {
    console.warn(`Warning: checksum verification skipped (${e.message})`);
  }

  fs.mkdirSync(binDir, { recursive: true });
  fs.writeFileSync(dest, binary);
  fs.chmodSync(dest, 0o755);
  console.log("konductor binary installed.");

  // Install agents and skills to ~/.kiro/
  installKiroAssets();
}

function copyDirSync(src, dest, force) {
  if (!fs.existsSync(src)) return;
  fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDirSync(srcPath, destPath, force);
    } else if (force || !fs.existsSync(destPath)) {
      fs.copyFileSync(srcPath, destPath);
    }
  }
}

function installKiroAssets() {
  const home = process.env.HOME || process.env.USERPROFILE;
  if (!home) return;
  const kiroDir = path.join(home, ".kiro");
  const pkgRoot = path.join(__dirname, "..");

  const assets = [
    { src: "agents", dest: "agents" },
    { src: "skills", dest: "skills" },
  ];

  for (const { src, dest } of assets) {
    const srcDir = path.join(pkgRoot, src);
    if (fs.existsSync(srcDir)) {
      copyDirSync(srcDir, path.join(kiroDir, dest), false);
      console.log(`Installed ${src} to ${path.join(kiroDir, dest)}`);
    }
  }
}

main().catch((err) => {
  console.warn(`Warning: failed to download konductor binary: ${err.message}`);
  console.warn("The binary will need to be installed manually. See: https://github.com/bnusunny/konductor");
  process.exit(0); // Don't fail npm install
});
