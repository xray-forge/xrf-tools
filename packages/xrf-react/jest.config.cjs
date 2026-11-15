const path = require("node:path");

const { createJestConfig } = require("@xrf/toolchain/jest");

/**
 * The workspace base, compiling TSX under jsdom with the gaps jsdom leaves filled, and the package's own setup.
 */
module.exports = createJestConfig({
  rootDir: __dirname,
  hasJsx: true,
  setupFiles: [require.resolve("@xrf/toolchain/jest-jsdom-globals")],
  setupFilesAfterEnv: [path.resolve(__dirname, "./test/jest-setup.ts")],
  testEnvironment: "jsdom",
});
