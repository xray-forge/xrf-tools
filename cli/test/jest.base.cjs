const path = require("node:path");

/**
 * Jest configuration every TypeScript member of the workspace starts from.
 *
 * @param {{ rootDir: string, hasDecorators?: boolean } & import("jest").Config} options - The member's root, whether
 *   its source uses TypeScript's legacy decorators, and its overrides.
 * @returns {import("jest").Config} The member's configuration.
 */
function createJestConfig({ rootDir, hasDecorators = false, ...overrides }) {
  return {
    cacheDirectory: "<rootDir>/target/jest-cache",
    clearMocks: true,
    coverageDirectory: "<rootDir>/target/coverage-report",
    coveragePathIgnorePatterns: ["/node_modules/"],
    maxWorkers: "50%",
    rootDir,
    roots: ["<rootDir>"],
    testEnvironment: "node",
    testMatch: ["<rootDir>/src/**/*.test.{ts,tsx}"],
    transform: {
      "^.+\\.[jt]sx?$": path.resolve(
        __dirname,
        hasDecorators ? "./swc-decorators-transformer.cjs" : "./swc-transformer.cjs"
      ),
    },
    verbose: true,
    workerIdleMemoryLimit: "512MB",
    ...overrides,
  };
}

module.exports = { createJestConfig };
