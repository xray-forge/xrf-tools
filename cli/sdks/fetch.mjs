import * as cp from "node:child_process";
import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import * as process from "node:process";

/**
 * Fetches the SDKs the native features build against into `.sdks/`, which `.cargo/config.toml` points the builds at:
 * each pinned to a commit and checked out sparsely, only the headers and libraries a build reads. It also places the
 * libraries taken from a pinned archive: Steam Audio for the audio on every platform, and on Windows libclang for the
 * builds and DXC for the renderer. Elsewhere `.sdks/libclang` links the system's libclang.
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
    // DLSS 310.9.16: the NGX headers, the static libraries the `dlss` feature links, and the runtime libraries a build
    // places beside its executable: super resolution's and ray reconstruction's.
    name: "dlss",
    repository: "https://github.com/NVIDIA/DLSS.git",
    commit: "9981c6350dc58c0ce21f024fd65e31e851c31689",
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
        "/lib/Linux_x86_64/rel/libnvidia-ngx-dlss.so.310.9.16",
        "/lib/Linux_x86_64/rel/libnvidia-ngx-dlssd.so.310.9.16",
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
 * @typedef {object} IDownloadedFile
 * @property {string} url - where it is downloaded from as it is
 * @property {string} file - its name under the tool's directory
 * @property {string} hash - its pinned SHA-256
 */

/**
 * @typedef {object} IArchivedTool
 * @property {string} name - its directory under `.sdks/`
 * @property {string} url - the archive it is taken from
 * @property {string} hash - the archive's pinned SHA-256
 * @property {Partial<Record<string, ReadonlyArray<IArchivedFile>>>} files - what is taken out of the archive on each
 *   `process.platform`; a tool with nothing for the platform is not fetched there
 * @property {ReadonlyArray<IDownloadedFile>} [downloads] - files fetched beside the archive's on every platform it is
 */

/** @type {ReadonlyArray<IArchivedTool>} */
const ARCHIVED_TOOLS = [
  {
    // libclang for Windows as LLVM builds it, from its Python package: bindgen loads it to generate the NGX and Steam
    // Audio bindings, and `.cargo/config.toml` points `LIBCLANG_PATH` at it, so no LLVM install is needed.
    name: "libclang",
    url:
      "https://files.pythonhosted.org/packages/0b/2d/3f480b1e1d31eb3d6de5e3ef641954e5c67430d5ac93b7fa7e07589576c7/" +
      "libclang-18.1.1-py2.py3-none-win_amd64.whl",
    hash: "4dd2d3b82fab35e2bf9ca717d7b63ac990a3519c7e312f19fa8e86dcc712f7fb",
    files: {
      win32: [
        {
          member: "libclang-18.1.1.data/platlib/clang/native/libclang.dll",
          file: "libclang.dll",
          hash: "b13ec96d9a036d6c6c36dd09e00415ead7acc7fcfd40e721bf33ea86b3490f9d",
        },
      ],
    },
  },
  {
    // DXC 1.9.2609 (September 2026), which the renderer's D3D12 compiles shader model 6 with, ray queries among it; the
    // application ships it beside its executable, as `bin/xrf-app/app/tauri.windows.conf.json` maps it, with its licences.
    name: "dxc",
    url: "https://github.com/microsoft/DirectXShaderCompiler/releases/download/v1.9.2609/dxc_2026_09_29.zip",
    hash: "ad31b1fc8443175d204f77a611fdb3ef2ec42759bdc2f1167368de24a4a7e7f1",
    files: {
      win32: [
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
  },
  {
    // Steam Audio 4.8.1 (February 2026), the audio's acoustics: `audionimbus-sys` links the library (`STEAMAUDIO_LIB_DIR`
    // in `.cargo/config.toml`), and the application ships it with Valve's licence and the third-party notices of what
    // the library is built with, as `bin/xrf-app/app/tauri.windows.conf.json` and `tauri.macos.conf.json` map them.
    name: "steam-audio",
    url: "https://github.com/ValveSoftware/steam-audio/releases/download/v4.8.1/steamaudio_4.8.1.zip",
    hash: "4a0aa5ec1176f38f0b0993a37c2259d9e86f27e22d5e24f83ec4c3cb9a1d5449",
    files: {
      win32: [
        {
          member: "steamaudio/lib/windows-x64/phonon.dll",
          file: "phonon.dll",
          hash: "ca3dbc01dbc24492717011e80f6a51404ca143ae344ca660971d2c983f1e058d",
        },
        {
          member: "steamaudio/lib/windows-x64/phonon.lib",
          file: "phonon.lib",
          hash: "b6b2dc8c5f91b4d7ae8d9eef03203df8d4675f1f8816fb7ac1b7a6ee3eb0f4d8",
        },
        {
          member: "steamaudio/THIRDPARTY.md",
          file: "THIRDPARTY.md",
          hash: "a95e143e7d2466e82a28d6680c7287d9fc7d772136c323ab538a6d24472e70f0",
        },
      ],
      darwin: [
        {
          member: "steamaudio/lib/osx/libphonon.dylib",
          file: "libphonon.dylib",
          hash: "7cedf26c7e1fdb378971989236c52af6723246226381f14da855a028f142f92c",
        },
        {
          member: "steamaudio/THIRDPARTY.md",
          file: "THIRDPARTY.md",
          hash: "a95e143e7d2466e82a28d6680c7287d9fc7d772136c323ab538a6d24472e70f0",
        },
      ],
      linux: [
        {
          member: "steamaudio/lib/linux-x64/libphonon.so",
          file: "libphonon.so",
          hash: "faf7c4d69e0f104366f6553cda38d3f946f761f7db223baf12663f6fbf2387da",
        },
        {
          member: "steamaudio/THIRDPARTY.md",
          file: "THIRDPARTY.md",
          hash: "a95e143e7d2466e82a28d6680c7287d9fc7d772136c323ab538a6d24472e70f0",
        },
      ],
    },
    downloads: [
      {
        url: "https://raw.githubusercontent.com/ValveSoftware/steam-audio/0da18255cca520771f363ee01f100572b39a308e/LICENSE.md",
        file: "LICENSE.md",
        hash: "cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30",
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
 * @param {string} url - what to download
 * @param {string} what - what it is, for an error
 * @returns {Promise<Buffer>} its bytes
 */
async function download(url, what) {
  const response = await fetch(url);

  if (!response.ok) {
    throw new Error(`${what} is not fetched: ${response.status} ${response.statusText}`);
  }

  return Buffer.from(await response.arrayBuffer());
}

/**
 * Takes `members` out of the zip `archive` into `into`: with Windows' own tar, the system's bsdtar on macOS, and unzip
 * on Linux, whose GNU tar reads no zip.
 *
 * @param {string} archive - the zip
 * @param {string} into - the directory the members land in, by their paths
 * @param {ReadonlyArray<string>} members - what to take out
 */
function extract(archive, into, members) {
  switch (process.platform) {
    case "win32":
      cp.execFileSync(path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe"), [
        "-xf",
        archive,
        "-C",
        into,
        ...members,
      ]);
      break;
    case "darwin":
      cp.execFileSync("/usr/bin/tar", ["-xf", archive, "-C", into, ...members]);
      break;
    default:
      cp.execFileSync("unzip", ["-q", "-o", archive, ...members, "-d", into]);
  }
}

/**
 * Places a tool's files for this platform in `.sdks/<name>/`, downloading its archive only where a file there is
 * missing or not the pinned one; a tool with nothing for this platform is left alone.
 *
 * @param {IArchivedTool} tool - the tool
 */
async function fetchArchived(tool) {
  const dir = path.join(SDKS_ROOT, tool.name);
  const files = tool.files[process.platform] ?? [];
  const downloads = tool.downloads ?? [];

  if (files.length === 0) {
    return;
  }

  function isPlaced(file) {
    return fs.existsSync(path.join(dir, file.file)) && hash(path.join(dir, file.file)) === file.hash;
  }

  if (!files.every(isPlaced)) {
    console.info(`Fetching ${tool.name} from ${tool.url}`);

    const bytes = await download(tool.url, `${tool.name}'s archive`);

    if (hash(bytes) !== tool.hash) {
      throw new Error(`${tool.name}'s archive is not the pinned one`);
    }

    const scratch = fs.mkdtempSync(path.join(os.tmpdir(), `xrf-${tool.name}-`));

    try {
      const archive = path.join(scratch, "archive.zip");

      fs.writeFileSync(archive, bytes);
      extract(
        archive,
        scratch,
        files.map((file) => file.member)
      );
      fs.mkdirSync(dir, { recursive: true });

      for (const file of files) {
        fs.copyFileSync(path.join(scratch, file.member), path.join(dir, file.file));
      }
    } finally {
      fs.rmSync(scratch, { recursive: true, force: true });
    }
  }

  for (const file of downloads.filter((it) => !isPlaced(it))) {
    console.info(`Fetching ${tool.name}'s ${file.file} from ${file.url}`);
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(path.join(dir, file.file), await download(file.url, `${tool.name}'s ${file.file}`));
  }

  for (const file of [...files, ...downloads]) {
    if (!isPlaced(file)) {
      throw new Error(`${tool.name}'s ${file.file} is not the pinned one`);
    }
  }

  console.info(`${tool.name}: ${path.relative(REPOSITORY_ROOT, dir)}`);
}

/**
 * The system's libclang, which bindgen loads to generate the NGX and Steam Audio bindings where no libclang is fetched.
 *
 * @returns {string | null} the library's path, or null where none is installed
 */
function findSystemLibclang() {
  const versioned = fs.existsSync("/usr/lib")
    ? fs
        .readdirSync("/usr/lib")
        .filter((name) => name.startsWith("llvm-"))
        .map((name) => path.join("/usr/lib", name, "lib"))
    : [];
  const xcode = readCommand("xcode-select", ["--print-path"]);
  const candidates = [
    readCommand("llvm-config", ["--libdir"]),
    xcode && path.join(xcode, "usr", "lib"),
    xcode && path.join(xcode, "Toolchains", "XcodeDefault.xctoolchain", "usr", "lib"),
    "/Library/Developer/CommandLineTools/usr/lib",
    "/opt/homebrew/opt/llvm/lib",
    "/usr/local/opt/llvm/lib",
    "/usr/lib",
    "/usr/lib/x86_64-linux-gnu",
    "/usr/local/lib",
    ...versioned,
  ].filter(Boolean);
  // Debian and Ubuntu install a versioned library, `libclang-<n>.so` or `libclang.so.<n>`, which clang-sys finds too.
  const pattern = /^libclang(-[\d.]+)?\.(so(\.[\d.]+)?|dylib)$/;

  for (const dir of candidates) {
    const name = fs.existsSync(dir) ? fs.readdirSync(dir).find((it) => pattern.test(it)) : undefined;

    if (name) {
      return path.join(dir, name);
    }
  }

  return null;
}

/**
 * Links the system's libclang into `.sdks/libclang`, where `.cargo/config.toml` points `LIBCLANG_PATH` on every
 * platform; one set in the environment wins over both.
 */
function linkSystemLibclang() {
  const dir = path.join(SDKS_ROOT, "libclang");
  const library = findSystemLibclang();

  if (library === null) {
    console.warn(
      "libclang is not found: building xrf-tools needs it installed (Xcode's command line tools, libclang-dev or " +
        "LLVM), or LIBCLANG_PATH set to the directory holding it."
    );

    return;
  }

  const link = path.join(dir, path.basename(library));

  fs.rmSync(dir, { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
  fs.symlinkSync(library, link);
  console.info(`libclang: ${path.relative(REPOSITORY_ROOT, link)} -> ${library}`);
}

/**
 * @param {string} command - a command
 * @param {ReadonlyArray<string>} args - its arguments
 * @returns {string | null} what it printed, trimmed, or null where it is not installed or fails
 */
function readCommand(command, args) {
  try {
    return cp.execFileSync(command, args, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim();
  } catch {
    return null;
  }
}

for (const sdk of SDKS) {
  fetchSdk(sdk);
}

for (const tool of ARCHIVED_TOOLS) {
  await fetchArchived(tool);
}

if (process.platform !== "win32") {
  linkSystemLibclang();
}
