import * as cp from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import * as process from "node:process";
import { parseArgs } from "node:util";

import { findCrateNotices, renderCrateLicenses, renderCrateNotices, REPOSITORY_ROOT, writeText } from "./licenses.mjs";

/**
 * Packs a built CLI into the archive it ships as: one `xrf-cli/` folder with the executable, the repository licence and
 * `licenses.txt`, the notices of every crate compiled into it. Windows gets a zip, Linux a gzipped tarball that keeps
 * the executable bit.
 *
 * Usage: node cli/package/cli.mjs [--profile release] [--name xrf-cli-windows-x64]
 * Writes `target/package/<name>.zip` on Windows and `target/package/<name>.tar.gz` on Linux.
 */
const WINDOWS = process.platform === "win32";
const EXECUTABLE = WINDOWS ? "xrf-cli.exe" : "xrf-cli";
const FOLDER = "xrf-cli";

if (!WINDOWS && process.platform !== "linux") {
  throw new Error(`The CLI is packaged on Windows and Linux only, not '${process.platform}'`);
}

const { values } = parseArgs({
  options: {
    profile: { type: "string", default: "release" },
    name: { type: "string", default: WINDOWS ? "xrf-cli-windows-x64" : "xrf-cli-linux-x64" },
  },
});

const executable = path.join(REPOSITORY_ROOT, "target", values.profile, EXECUTABLE);

if (!fs.existsSync(executable)) {
  throw new Error(`'${path.relative(REPOSITORY_ROOT, executable)}' is missing: build the CLI first`);
}

const graph = { binary: "xrf-cli" };
const output = path.join(REPOSITORY_ROOT, "target", "package");
const staging = path.join(output, values.name);
const folder = path.join(staging, FOLDER);
const archive = path.join(output, `${values.name}${WINDOWS ? ".zip" : ".tar.gz"}`);

fs.rmSync(staging, { recursive: true, force: true });
fs.rmSync(archive, { force: true });
fs.mkdirSync(folder, { recursive: true });
fs.copyFileSync(executable, path.join(folder, EXECUTABLE));
fs.copyFileSync(path.join(REPOSITORY_ROOT, "LICENSE"), path.join(folder, "LICENSE"));
writeText(
  path.join(folder, "licenses.txt"),
  [renderCrateLicenses(graph), renderCrateNotices(findCrateNotices(graph))].filter(Boolean).join("\n")
);

if (WINDOWS) {
  // Windows' own tar writes a zip for an archive named `.zip`.
  cp.execFileSync(
    path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe"),
    ["-a", "-c", "-f", archive, "-C", staging, FOLDER],
    { stdio: "inherit" }
  );
} else {
  cp.execFileSync("tar", ["-czf", archive, "-C", staging, FOLDER], { stdio: "inherit" });
}

fs.rmSync(staging, { recursive: true, force: true });

const megabytes = (fs.statSync(archive).size / (1024 * 1024)).toFixed(1);

console.info(`${path.relative(REPOSITORY_ROOT, archive)}: ${megabytes} MB`);
