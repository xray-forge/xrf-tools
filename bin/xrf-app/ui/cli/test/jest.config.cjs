const path = require("node:path");

const { createJestConfig } = require("@xrf/toolchain/jest");

const ROOT_DIR = path.resolve(__dirname, "../../");

/**
 * Jest configuration for the desktop frontend: the workspace base, plus jsdom, the application's own transforms and
 * its setup.
 *
 * @type {import('jest').Config}
 */
module.exports = createJestConfig({
  rootDir: ROOT_DIR,
  coveragePathIgnorePatterns: ["/node_modules/", "/fixtures/"],
  globals: {
    __REPOSITORY_URL__: require(path.join(ROOT_DIR, "package.json")).repository.url,
  },
  moduleNameMapper: {
    "\\.(css|less|svg|png|jpg|woff2?)$": require.resolve("@xrf/toolchain/jest-asset-stub"),
    "^@/(.*)$": "<rootDir>/src/$1",
  },
  setupFiles: [require.resolve("@xrf/toolchain/jest-jsdom-globals")],
  setupFilesAfterEnv: [path.resolve(__dirname, "./jest-setup.ts")],
  testEnvironment: "jsdom",
  transform: {
    "^.+\\.[jt]sx?$": path.resolve(__dirname, "./transformer.cjs"),
  },
});
