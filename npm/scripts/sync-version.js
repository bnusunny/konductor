#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const version = fs.readFileSync(path.join(__dirname, "..", "..", "version.txt"), "utf8").trim();
const pkgPath = path.join(__dirname, "..", "package.json");
const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
pkg.version = version;
fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
console.log(`Synced version to ${version}`);
