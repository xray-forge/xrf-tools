import * as cp from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

/**
 * The third-party notices a shipped binary owes, gathered from what it actually links: the registry crates through
 * `cargo about` (the accepted licences are in `about.toml`), and the credits a workspace crate keeps in its own
 * directory for code it adapts, as `THIRD_PARTY_NOTICES.md` or `LICENSE-*` files.
 */
export const REPOSITORY_ROOT = path.resolve(import.meta.dirname, "..", "..");

const RULE = "=".repeat(80);
const THIN_RULE = "-".repeat(80);
const LINE_WIDTH = 120;
/** @type {Record<string, string>} */
const MANIFESTS = { "xrf-app": "bin/xrf-app/app/Cargo.toml", "xrf-cli": "bin/xrf-cli/Cargo.toml" };
const NOTICE_FILE = /^(THIRD_PARTY_NOTICES\.md|LICENSE-.+)$/;
// `cargo tree --format {p}` prints a workspace member with its directory, `name v0.1.0 (proc-macro) (C:\...)`, and
// marks a repeated crate `(*)`.
const LOCAL_PACKAGE = /^(\S+) v\S+(?: \(proc-macro\))? \(([A-Za-z]:\\[^)]+|\/[^)]+)\)(?: \(\*\))?$/;

/**
 * @typedef {object} BinaryGraph
 * @property {string} binary - The workspace package that builds the binary.
 * @property {Array<string>} [features] - The features the shipped build enables.
 */

/**
 * @typedef {object} CrateNotice
 * @property {string} crate - The workspace crate that keeps the file.
 * @property {string} file - The file's path.
 * @property {string} name - The name it ships under: the crate's name, then the file's, with `.txt` added to a bare
 *   `LICENSE-*`.
 */

/**
 * @param {BinaryGraph} graph - The binary and its features.
 * @returns {string} Every registry crate the binary links, grouped by licence text, with each text in full.
 */
export function renderCrateLicenses({ binary, features = [] }) {
  // A file, not stdout: under PowerShell, the CI shell on Windows, cargo-about refuses to write to a pipe.
  const output = path.join(REPOSITORY_ROOT, "target", "licenses", `${binary}-crates.json`);

  fs.mkdirSync(path.dirname(output), { recursive: true });
  cp.execFileSync(
    "cargo",
    [
      "about",
      "generate",
      "--format",
      "json",
      "--locked",
      "--fail",
      "--manifest-path",
      getManifest(binary),
      "--target",
      getHostTarget(),
      "--output-file",
      output,
      ...(features.length > 0 ? ["--features", features.join(" ")] : []),
    ],
    { cwd: REPOSITORY_ROOT, stdio: ["ignore", "inherit", "inherit"] }
  );

  const report = JSON.parse(readText(output));

  const sections = report.licenses.map((license) => {
    const crates = license.used_by.map((it) => `${it.crate.name} ${it.crate.version}`).join(", ");

    return [RULE, license.name, ...wrap(`Used by: ${crates}`), THIN_RULE, license.text.trim(), ""].join("\n");
  });

  return [`Third-party crates linked into ${binary}, grouped by the licence each is used under.`, "", ...sections].join(
    "\n"
  );
}

/**
 * @param {BinaryGraph} graph - The binary and its features.
 * @returns {Array<CrateNotice>} The credits each linked workspace crate keeps for code it adapts.
 */
export function findCrateNotices({ binary, features = [] }) {
  const tree = cp.execFileSync(
    "cargo",
    [
      "tree",
      "--locked",
      "--package",
      binary,
      "--edges",
      "normal",
      "--target",
      getHostTarget(),
      "--prefix",
      "none",
      "--format",
      "{p}",
      ...(features.length > 0 ? ["--features", features.join(",")] : []),
    ],
    { cwd: REPOSITORY_ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] }
  );

  /** @type {Map<string, string>} */
  const members = new Map();

  for (const line of tree.split(/\r?\n/)) {
    const match = LOCAL_PACKAGE.exec(line.trim());

    if (match) {
      members.set(match[1], match[2]);
    }
  }

  return [...members]
    .sort(([left], [right]) => left.localeCompare(right))
    .flatMap(([crate, directory]) =>
      fs
        .readdirSync(directory)
        .filter((file) => NOTICE_FILE.test(file))
        .sort()
        .map((file) => ({
          crate,
          file: path.join(directory, file),
          name: `${crate}-${path.extname(file) ? file : `${file}.txt`}`,
        }))
    );
}

/**
 * @param {Array<CrateNotice>} notices - Workspace crates' credits.
 * @returns {string} Each credit in full, under the crate and file it comes from.
 */
export function renderCrateNotices(notices) {
  return notices
    .map((notice) =>
      [RULE, `${notice.crate}: ${path.basename(notice.file)}`, THIN_RULE, readText(notice.file).trim(), ""].join("\n")
    )
    .join("\n");
}

/**
 * @param {string} file - The JSON the UI build writes beside its bundle (`bin/xrf-app/ui/cli/build/licenses.ts`).
 * @returns {string} Every npm package the UI bundle includes, with its licence files in full.
 */
export function renderNpmLicenses(file) {
  if (!fs.existsSync(file)) {
    throw new Error(`'${path.relative(REPOSITORY_ROOT, file)}' is missing: build the user interface first`);
  }

  /** @type {Array<{ name: string, version: string, license: string, files: Array<{ name: string, text: string }> }>} */
  const packages = JSON.parse(readText(file));

  const sections = packages.map((it) =>
    [
      RULE,
      `${it.name} ${it.version} (${it.license})`,
      ...it.files.flatMap((licence) => [THIN_RULE, licence.text.trim()]),
      "",
    ].join("\n")
  );

  return ["Third-party npm packages bundled into the xrf-app user interface.", "", ...sections].join("\n");
}

/**
 * @param {string} file - A text file.
 * @param {string} text - Its contents.
 */
export function writeText(file, text) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, text.replace(/\r\n/g, "\n"));
}

/**
 * @param {string} file - A text file.
 * @returns {string} Its contents, without a byte-order mark.
 */
function readText(file) {
  return fs.readFileSync(file, "utf8").replace(/^\uFEFF/, "");
}

/**
 * @param {string} binary - A workspace package that builds a binary.
 * @returns {string} Its manifest.
 */
function getManifest(binary) {
  const manifest = MANIFESTS[binary];

  if (!manifest) {
    throw new Error(`No shipped binary is called '${binary}'`);
  }

  return path.join(REPOSITORY_ROOT, manifest);
}

/** @type {string | null} */
let hostTarget = null;

/**
 * @returns {string} The target triple of the toolchain, which the binary is built for.
 */
function getHostTarget() {
  if (hostTarget === null) {
    const host = cp
      .execFileSync("rustc", ["-vV"], { cwd: REPOSITORY_ROOT, encoding: "utf8" })
      .split(/\r?\n/)
      .find((line) => line.startsWith("host: "));

    if (!host) {
      throw new Error("'rustc -vV' named no host target");
    }

    hostTarget = host.slice("host: ".length).trim();
  }

  return hostTarget;
}

/**
 * @param {string} text - One line of prose.
 * @returns {Array<string>} It broken at spaces into lines no wider than the notices' width.
 */
function wrap(text) {
  /** @type {Array<string>} */
  const lines = [];
  let line = "";

  for (const word of text.split(" ")) {
    if (line && line.length + 1 + word.length > LINE_WIDTH) {
      lines.push(line);
      line = word;
    } else {
      line = line ? `${line} ${word}` : word;
    }
  }

  return line ? [...lines, line] : lines;
}
