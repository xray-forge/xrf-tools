import * as cp from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import * as process from "node:process";
import { parseArgs } from "node:util";

/**
 * Packs a built desktop application into the archive it ships as. The archive holds one `xrf-app/` folder with the
 * executable, the renderer resources beside it (the shader compiler, lookup textures and, in DLSS builds, NVIDIA's runtime
 * and licence) and the repository licence.
 *
 * Usage: node cli/package/app.mjs [--profile release] [--name xrf-app-windows-x64]
 * Writes `target/package/<name>.zip`.
 */
const REPOSITORY_ROOT = path.resolve(import.meta.dirname, "..", "..");
// What a Git LFS pointer starts with: a checkout without `git lfs pull` leaves one in place of each binary.
const LFS_POINTER = Buffer.from("version https://git-lfs.github.com/spec/v1");
const EXECUTABLE = "xrf-app.exe";
const FOLDER = "xrf-app";

const { values } = parseArgs({
  options: {
    profile: { type: "string", default: "release" },
    name: { type: "string", default: "xrf-app-windows-x64" },
  },
});

if (process.platform !== "win32") {
  throw new Error("The desktop application is packaged on Windows only");
}

const built = path.join(REPOSITORY_ROOT, "target", values.profile);
const executable = path.join(built, EXECUTABLE);
const resources = path.join(built, "resources");

for (const required of [executable, path.join(resources, "renderer")]) {
  if (!fs.existsSync(required)) {
    throw new Error(`'${path.relative(REPOSITORY_ROOT, required)}' is missing: build the application first`);
  }
}

const pointers = fs
  .readdirSync(resources, { recursive: true, withFileTypes: true })
  .filter((entry) => entry.isFile())
  .map((entry) => path.join(entry.parentPath, entry.name))
  .filter((file) => isLfsPointer(file));

if (pointers.length > 0) {
  throw new Error(
    `Git LFS pointers stand in for resources, so run 'git lfs pull' and build again: ${pointers
      .map((file) => path.relative(REPOSITORY_ROOT, file))
      .join(", ")}`
  );
}

const output = path.join(REPOSITORY_ROOT, "target", "package");
const staging = path.join(output, values.name);
const folder = path.join(staging, FOLDER);
const archive = path.join(output, `${values.name}.zip`);

fs.rmSync(staging, { recursive: true, force: true });
fs.rmSync(archive, { force: true });
fs.mkdirSync(folder, { recursive: true });
fs.copyFileSync(executable, path.join(folder, EXECUTABLE));
fs.cpSync(resources, path.join(folder, "resources"), { recursive: true });
fs.copyFileSync(path.join(REPOSITORY_ROOT, "LICENSE"), path.join(folder, "LICENSE"));

// Windows' own tar writes a zip for an archive named `.zip`.
cp.execFileSync(
  path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe"),
  ["-a", "-c", "-f", archive, "-C", staging, FOLDER],
  { stdio: "inherit" }
);
fs.rmSync(staging, { recursive: true, force: true });

const megabytes = (fs.statSync(archive).size / (1024 * 1024)).toFixed(1);
const isDlss = fs.existsSync(path.join(resources, "renderer", "dlss"));

console.info(`${path.relative(REPOSITORY_ROOT, archive)}: ${megabytes} MB${isDlss ? ", with DLSS" : ", without DLSS"}`);

/**
 * @param {string} file - A packed resource.
 * @returns {boolean} Whether it is a Git LFS pointer rather than the file it names.
 */
function isLfsPointer(file) {
  const head = Buffer.alloc(LFS_POINTER.length);
  const descriptor = fs.openSync(file, "r");

  try {
    return fs.readSync(descriptor, head, 0, head.length, 0) === head.length && head.equals(LFS_POINTER);
  } finally {
    fs.closeSync(descriptor);
  }
}
