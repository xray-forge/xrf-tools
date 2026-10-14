import * as cp from "node:child_process";
import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import * as process from "node:process";

/**
 * Fetches the SDKs the optional native features build against into `.sdks/`, which `.cargo/config.toml` points the
 * builds at: each pinned to a commit and checked out sparsely, only the headers and libraries a build reads.
 */
const REPOSITORY_ROOT = path.resolve(import.meta.dirname, "..", "..");
const SDKS_ROOT = path.join(REPOSITORY_ROOT, ".sdks");

/**
 * @typedef {object} ISdk
 * @property {string} name - its directory under `.sdks/`
 * @property {string} repository - where it is fetched from
 * @property {string} commit - the commit it is pinned to
 * @property {Record<string, ReadonlyArray<string>>} paths - what is checked out, on every platform (`common`) and on
 *   each `process.platform`
 */

/** @type {ReadonlyArray<ISdk>} */
const SDKS = [
  {
    // DLSS 310.9.1: the NGX headers, the static libraries the `dlss` feature links, and the runtime libraries a build
    // places beside its executable: super resolution's and ray reconstruction's.
    name: "dlss",
    repository: "https://github.com/NVIDIA/DLSS.git",
    commit: "374959484e79a640feaba44c93ac8cfb0a03f5b5",
    paths: {
      common: ["/include/", "/LICENSE.txt"],
      win32: [
        "/lib/Windows_x86_64/x64/nvsdk_ngx_d.lib",
        "/lib/Windows_x86_64/x64/nvsdk_ngx_s.lib",
        "/lib/Windows_x86_64/rel/nvngx_dlss.dll",
        "/lib/Windows_x86_64/rel/nvngx_dlssd.dll",
      ],
      linux: [
        "/lib/Linux_x86_64/libnvsdk_ngx.a",
        "/lib/Linux_x86_64/rel/libnvidia-ngx-dlss.so.310.9.1",
        "/lib/Linux_x86_64/rel/libnvidia-ngx-dlssd.so.310.9.1",
      ],
    },
  },
  {
    // Vulkan's C headers 1.4.365, which the NGX headers include: what of the Vulkan SDK the `dlss` feature needs.
    name: "vulkan-headers",
    repository: "https://github.com/KhronosGroup/Vulkan-Headers.git",
    commit: "c46850864f4661461b0f6cb9922c058ffea4915e",
    paths: { common: ["/include/vulkan/*.h", "/include/vk_video/", "/LICENSE.md"] },
  },
];

/**
 * @param {string} dir - a repository
 * @param {ReadonlyArray<string>} args - git's arguments
 * @returns {string} what git printed, trimmed
 */
function git(dir, args) {
  return cp
    .execFileSync("git", ["-C", dir, ...args], { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] })
    .trim();
}

/**
 * @param {string} dir - a repository
 * @returns {string | null} the commit checked out, or null where none is
 */
function readHead(dir) {
  try {
    return cp
      .execFileSync("git", ["-C", dir, "rev-parse", "HEAD"], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] })
      .trim();
  } catch {
    return null;
  }
}

/**
 * Checks an SDK out at its commit, fetching only the blobs its paths need; one already there is left as it is.
 *
 * @param {ISdk} sdk - the SDK
 */
function fetchSdk(sdk) {
  const dir = path.join(SDKS_ROOT, sdk.name);
  const patterns = [...sdk.paths.common, ...(sdk.paths[process.platform] ?? [])];

  fs.mkdirSync(dir, { recursive: true });

  if (!fs.existsSync(path.join(dir, ".git"))) {
    git(dir, ["init", "--quiet"]);
    git(dir, ["remote", "add", "origin", sdk.repository]);
  }

  git(dir, ["remote", "set-url", "origin", sdk.repository]);
  git(dir, ["sparse-checkout", "set", "--no-cone", ...patterns]);

  if (readHead(dir) !== sdk.commit) {
    console.info(`Fetching ${sdk.name} at ${sdk.commit.slice(0, 12)} from ${sdk.repository}`);
    git(dir, ["fetch", "--quiet", "--depth", "1", "--filter=blob:none", "origin", sdk.commit]);
    git(dir, ["checkout", "--quiet", "--detach", sdk.commit]);
  }

  if (readHead(dir) !== sdk.commit) {
    throw new Error(`${sdk.name} is not at ${sdk.commit} in ${dir}`);
  }

  console.info(`${sdk.name}: ${path.relative(REPOSITORY_ROOT, dir)} at ${sdk.commit.slice(0, 12)}`);
}

/**
 * libclang for Windows as LLVM builds it, from its Python package: bindgen loads it to generate the NGX bindings, and
 * `.cargo/config.toml` points `LIBCLANG_PATH` at it, so no LLVM install is needed.
 */
const LIBCLANG = {
  url:
    "https://files.pythonhosted.org/packages/0b/2d/3f480b1e1d31eb3d6de5e3ef641954e5c67430d5ac93b7fa7e07589576c7/" +
    "libclang-18.1.1-py2.py3-none-win_amd64.whl",
  wheel: "4dd2d3b82fab35e2bf9ca717d7b63ac990a3519c7e312f19fa8e86dcc712f7fb",
  member: "libclang-18.1.1.data/platlib/clang/native/libclang.dll",
  library: "b13ec96d9a036d6c6c36dd09e00415ead7acc7fcfd40e721bf33ea86b3490f9d",
};

/**
 * @param {Buffer | string} data - bytes, or a file's path
 * @returns {string} their SHA-256, in hex
 */
function hash(data) {
  return crypto
    .createHash("sha256")
    .update(typeof data === "string" ? fs.readFileSync(data) : data)
    .digest("hex");
}

/**
 * Places libclang in `.sdks/libclang/` on Windows, downloading its package only where the library there is missing or
 * not the pinned one.
 */
async function fetchLibclang() {
  const dir = path.join(SDKS_ROOT, "libclang");
  const library = path.join(dir, "libclang.dll");

  if (fs.existsSync(library) && hash(library) === LIBCLANG.library) {
    console.info(`libclang: ${path.relative(REPOSITORY_ROOT, library)}`);

    return;
  }

  console.info(`Fetching libclang from ${LIBCLANG.url}`);

  const response = await fetch(LIBCLANG.url);

  if (!response.ok) {
    throw new Error(`libclang's package is not fetched: ${response.status} ${response.statusText}`);
  }

  const wheel = Buffer.from(await response.arrayBuffer());

  if (hash(wheel) !== LIBCLANG.wheel) {
    throw new Error("libclang's package is not the pinned one");
  }

  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "xrf-libclang-"));

  try {
    const archive = path.join(scratch, "libclang.whl");

    fs.writeFileSync(archive, wheel);
    // Windows' own tar reads the package as the zip it is.
    cp.execFileSync(path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe"), [
      "-xf",
      archive,
      "-C",
      scratch,
      LIBCLANG.member,
    ]);
    fs.mkdirSync(dir, { recursive: true });
    fs.copyFileSync(path.join(scratch, LIBCLANG.member), library);
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }

  if (hash(library) !== LIBCLANG.library) {
    throw new Error("libclang's library is not the pinned one");
  }

  console.info(`libclang: ${path.relative(REPOSITORY_ROOT, library)}`);
}

/**
 * @returns {boolean} whether bindgen finds libclang, which generating the NGX bindings needs
 */
function hasLibclang() {
  const candidates = [process.env.LIBCLANG_PATH, "/usr/lib", "/usr/lib/x86_64-linux-gnu", "/usr/local/lib"].filter(
    Boolean
  );
  const library = "libclang.so";

  return candidates.some((dir) => fs.existsSync(path.join(dir, library)));
}

for (const sdk of SDKS) {
  fetchSdk(sdk);
}

if (process.platform === "win32") {
  await fetchLibclang();
} else if (!hasLibclang()) {
  console.warn(
    "libclang is not found: building with `--features dlss` needs it installed (libclang-dev or LLVM), or " +
      "LIBCLANG_PATH set to the directory holding it."
  );
}
