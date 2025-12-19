#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const changelogPath = path.join(__dirname, '../../docs/CHANGELOG.md');
const changelog = fs.readFileSync(changelogPath, 'utf8');

const version = process.env.GITHUB_REF_NAME || process.argv[2];

if (!version) {
  console.error('Error: No version provided. Use GITHUB_REF_NAME env var or pass as argument.');
  process.exit(1);
}

const cleanVersion = version.startsWith('v') ? version : `v${version}`;

const versionRegex = new RegExp(`^## \\[${cleanVersion}\\] - (\\d{4}-\\d{2}-\\d{2})`, 'm');
const match = changelog.match(versionRegex);

if (!match) {
  console.error(`Error: Version ${cleanVersion} not found in CHANGELOG.md`);
  process.exit(1);
}

const startIndex = match.index;
const endMarker = /^## \[/m;
const endMatch = endMarker.exec(changelog.slice(startIndex + match[0].length));

const endIndex = endMatch 
  ? startIndex + match[0].length + endMatch.index
  : changelog.length;

const releaseNotes = changelog.slice(startIndex, endIndex).trim();

if (process.env.GITHUB_OUTPUT) {
  const output = `release_notes<<EOF\n${releaseNotes}\nEOF\n`;
  fs.appendFileSync(process.env.GITHUB_OUTPUT, output);
  console.log(`✅ Release notes extracted for ${cleanVersion}`);
} else {
  console.log(releaseNotes);
}