const path = require("node:path");

const { createJestConfig } = require("@xrf/toolchain/jest");

/**
 * The workspace base, compiled with decorators for the session layer's services, and the package's own IPC mocks
 * standing in for Tauri and the loopback transport.
 */
module.exports = createJestConfig({
  rootDir: __dirname,
  setupFilesAfterEnv: [path.resolve(__dirname, "./test/jest-setup.ts")],
  transform: {
    "^.+\\.[jt]sx?$": path.resolve(__dirname, "./test/transformer.cjs"),
  },
});
