import * as cp from "node:child_process";
import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import * as process from "node:process";

/**
 * Fetches the SDKs the optional native features build against into `.sdks/`, which `.cargo/config.toml` points the
 * builds at: each pinned to a commit and checked out sparsely, only the headers and libraries a build reads. On Windows
 * it also places the libraries taken from a pinned archive: libclang for the builds and DXC for the renderer.
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
 * Checks an SDK out at its commit, fetching only the blobs its paths need; one already there is left as it is, and one
 * at another commit is fetched afresh, so no older commit's files ride along in a cache saved from it.
 *
 * @param {ISdk} sdk - the SDK
 */
function fetchSdk(sdk) {
  const dir = path.join(SDKS_ROOT, sdk.name);
  const patterns = [...sdk.paths.common, ...(sdk.paths[process.platform] ?? [])];
  const head = readHead(dir);

  if (head !== null && head !== sdk.commit) {
    console.info(`${sdk.name} moves from ${head.slice(0, 12)}, so it is fetched afresh`);
    fs.rmSync(dir, { recursive: true, force: true });
  }

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
 * @typedef {object} IArchivedFile
 * @property {string} member - where the archive holds it
 * @property {string} file - its name under the tool's directory
 * @property {string} hash - its pinned SHA-256
 */

/**
 * @typedef {object} IArchivedTool
 * @property {string} name - its directory under `.sdks/`
 * @property {string} url - the archive it is taken from
 * @property {string} hash - the archive's pinned SHA-256
 * @property {ReadonlyArray<IArchivedFile>} files - what is taken out of the archive
 */

/** @type {ReadonlyArray<IArchivedTool>} */
const ARCHIVED_TOOLS = [
  {
    // libclang for Windows as LLVM builds it, from its Python package: bindgen loads it to generate the NGX bindings,
    // and `.cargo/config.toml` points `LIBCLANG_PATH` at it, so no LLVM install is needed.
    name: "libclang",
    url:
      "https://files.pythonhosted.org/packages/0b/2d/3f480b1e1d31eb3d6de5e3ef641954e5c67430d5ac93b7fa7e07589576c7/" +
      "libclang-18.1.1-py2.py3-none-win_amd64.whl",
    hash: "4dd2d3b82fab35e2bf9ca717d7b63ac990a3519c7e312f19fa8e86dcc712f7fb",
    files: [
      {
        member: "libclang-18.1.1.data/platlib/clang/native/libclang.dll",
        file: "libclang.dll",
        hash: "b13ec96d9a036d6c6c36dd09e00415ead7acc7fcfd40e721bf33ea86b3490f9d",
      },
    ],
  },
  {
    // DXC 1.9.2609 (September 2026), which the renderer's D3D12 compiles shader model 6 with, ray queries among it; the
    // application ships it beside its executable, as `bin/xrf-app/app/tauri.windows.conf.json` maps it, with its licences.
    name: "dxc",
    url: "https://github.com/microsoft/DirectXShaderCompiler/releases/download/v1.9.2609/dxc_2026_09_29.zip",
    hash: "ad31b1fc8443175d204f77a611fdb3ef2ec42759bdc2f1167368de24a4a7e7f1",
    files: [
      {
        member: "bin/x64/dxcompiler.dll",
        file: "dxcompiler.dll",
        hash: "0689e2fee0a1c9df8662a59680469f24e2a3ad7ce6bbbcc9284f7a83172bfc59",
      },
      {
        member: "LICENCE-MIT.txt",
        file: "LICENCE-MIT.txt",
        hash: "903df5512f7d02609fed0c780a9b704f5a3eeb6e4d84ebe42a29845c81899a3c",
      },
      {
        member: "LICENSE-LLVM.txt",
        file: "LICENSE-LLVM.txt",
        hash: "729615317e28dd03907e46f0fc3b5e88f7853cee61d1a1471d2749335516b46f",
      },
      {
        member: "LICENSE-MS.txt",
        file: "LICENSE-MS.txt",
        hash: "734f72f239fe7b07b4c7203f294c1a7ce27095687278bab7e56d630d7c672963",
      },
    ],
  },
];

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
 * Places a tool's files in `.sdks/<name>/`, downloading its archive only where a file there is missing or not the
 * pinned one.
 *
 * @param {IArchivedTool} tool - the tool
 */
async function fetchArchived(tool) {
  const dir = path.join(SDKS_ROOT, tool.name);

  function isPlaced(file) {
    return fs.existsSync(path.join(dir, file.file)) && hash(path.join(dir, file.file)) === file.hash;
  }

  if (tool.files.every(isPlaced)) {
    console.info(`${tool.name}: ${path.relative(REPOSITORY_ROOT, dir)}`);

    return;
  }

  console.info(`Fetching ${tool.name} from ${tool.url}`);

  const response = await fetch(tool.url);

  if (!response.ok) {
    throw new Error(`${tool.name}'s archive is not fetched: ${response.status} ${response.statusText}`);
  }

  const bytes = Buffer.from(await response.arrayBuffer());

  if (hash(bytes) !== tool.hash) {
    throw new Error(`${tool.name}'s archive is not the pinned one`);
  }

  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), `xrf-${tool.name}-`));

  try {
    const archive = path.join(scratch, "archive.zip");

    fs.writeFileSync(archive, bytes);
    // Windows' own tar reads a zip, which a Python package is too.
    cp.execFileSync(path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe"), [
      "-xf",
      archive,
      "-C",
      scratch,
      ...tool.files.map((file) => file.member),
    ]);
    fs.mkdirSync(dir, { recursive: true });

    for (const file of tool.files) {
      fs.copyFileSync(path.join(scratch, file.member), path.join(dir, file.file));
    }
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }

  for (const file of tool.files) {
    if (!isPlaced(file)) {
      throw new Error(`${tool.name}'s ${file.file} is not the pinned one`);
    }
  }

  console.info(`${tool.name}: ${path.relative(REPOSITORY_ROOT, dir)}`);
}

/**
 * @returns {boolean} whether bindgen finds libclang, which generating the NGX bindings needs
 */
function hasLibclang() {
  const versioned = fs.existsSync("/usr/lib")
    ? fs
        .readdirSync("/usr/lib")
        .filter((name) => name.startsWith("llvm-"))
        .map((name) => path.join("/usr/lib", name, "lib"))
    : [];
  const candidates = [
    process.env.LIBCLANG_PATH,
    readLlvmLibdir(),
    "/usr/lib",
    "/usr/lib/x86_64-linux-gnu",
    "/usr/local/lib",
    ...versioned,
  ].filter(Boolean);

  // Debian and Ubuntu install a versioned library, `libclang-<n>.so` or `libclang.so.<n>`, which clang-sys finds too.
  return candidates.some(
    (dir) => fs.existsSync(dir) && fs.readdirSync(dir).some((name) => /^libclang(-[\d.]+)?\.so(\.[\d.]+)?$/.test(name))
  );
}

/**
 * @returns {string | null} the library directory `llvm-config` reports, or null where it is not installed
 */
function readLlvmLibdir() {
  try {
    return cp
      .execFileSync("llvm-config", ["--libdir"], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] })
      .trim();
  } catch {
    return null;
  }
}

for (const sdk of SDKS) {
  fetchSdk(sdk);
}

if (process.platform === "win32") {
  for (const tool of ARCHIVED_TOOLS) {
    await fetchArchived(tool);
  }
} else if (!hasLibclang()) {
  console.warn(
    "libclang is not found: building with `--features dlss` needs it installed (libclang-dev or LLVM), or " +
      "LIBCLANG_PATH set to the directory holding it."
  );
}
