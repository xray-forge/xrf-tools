import * as cp from "node:child_process";
import * as path from "node:path";
import * as process from "node:process";

import husky from "husky";

/**
 * Installs this package's git hooks, from `pnpm install`.
 */
const PACKAGE_ROOT = path.resolve(import.meta.dirname, "..", "..");

/**
 * @returns {string | null} absolute working tree root, or null when this is not a git checkout
 */
function resolveRepositoryRoot() {
  try {
    return cp
      .execFileSync("git", ["rev-parse", "--show-toplevel"], {
        cwd: PACKAGE_ROOT,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "ignore"],
      })
      .trim();
  } catch {
    return null;
  }
}

const repositoryRoot = resolveRepositoryRoot();

// Sources are also consumed unpacked from outside a checkout; that install must still succeed.
if (repositoryRoot) {
  process.chdir(repositoryRoot);

  const hooks = path
    .relative(repositoryRoot, path.join(PACKAGE_ROOT, "cli", ".husky"))
    .split(path.sep)
    .join("/");

  const failure = husky(hooks);

  // Never fail the install over this: a broken hook installation must not block CI or a first checkout.
  if (failure) {
    console.error(`xrf-ui: commit hooks are not installed (${failure}).`);
  }
}
