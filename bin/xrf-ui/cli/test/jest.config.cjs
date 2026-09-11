const path = require("node:path");

const ROOT_DIR = path.resolve(__dirname, "../../");

/**
 * Jest configuration for the desktop frontend.
 *
 * The config is `.cjs` rather than `.ts` because this package has no `ts-node`; adding one purely to
 * parse a config would be its own cost. Test authors see the same `@jest/globals` API either way.
 *
 * @type {import('jest').Config}
 */
module.exports = {
  cacheDirectory: "<rootDir>/target/jest-cache",
  clearMocks: true,
  coverageDirectory: "<rootDir>/target/coverage-report",
  coveragePathIgnorePatterns: ["/node_modules/", "<rootDir>/src/fixtures/"],
  globals: {
    __REPOSITORY_URL__: require(path.join(ROOT_DIR, "package.json")).repository.url,
  },
  moduleNameMapper: {
    "\\.(css|less|svg|png|jpg|woff2?)$": path.resolve(__dirname, "./asset-stub.cjs"),
    "^@/(.*)$": "<rootDir>/src/$1",
    // Exercise the same Three.js entry as Vite; r186 deprecates its CommonJS entry.
    "^three$": "<rootDir>/node_modules/three/build/three.module.js",
  },
  rootDir: ROOT_DIR,
  roots: ["<rootDir>"],
  setupFiles: [path.resolve(__dirname, "./jest-global.cjs")],
  setupFilesAfterEnv: [path.resolve(__dirname, "./jest-setup.ts")],
  testEnvironment: "jsdom",
  testMatch: ["<rootDir>/src/**/*.test.{ts,tsx}"],
  maxWorkers: "50%",
  transform: {
    "^.+\\.[jt]sx?$": path.resolve(__dirname, "./transformer.cjs"),
  },
  // Compile Three.js and its ESM addons together; leave other dependency runtimes intact.
  transformIgnorePatterns: ["^(?!.*/three/).*/node_modules/"],
  verbose: true,
  workerIdleMemoryLimit: "512MB",
};
