const path = require("node:path");

/** The workspace root, where the lockfile and `pnpm-workspace.yaml` live. */
const WORKSPACE_ROOT = path.resolve(__dirname, "..");

/** The workspace lockfile, which every transform cache key has to cover. */
const LOCKFILE = path.join(WORKSPACE_ROOT, "pnpm-lock.yaml");

module.exports = { LOCKFILE, WORKSPACE_ROOT };
