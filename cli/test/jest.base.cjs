const path = require("node:path");

/**
 * Picks the compile a member's source needs.
 *
 * @param {boolean} hasDecorators - Whether its source uses TypeScript's legacy decorators.
 * @param {boolean} hasJsx - Whether its source holds TSX components.
 * @returns {string} The transformer's path.
 */
function getTransformer(hasDecorators, hasJsx) {
  if (hasDecorators && hasJsx) {
    throw new Error("No transformer compiles both legacy decorators and JSX for a package; add one when needed.");
  }

  if (hasDecorators) {
    return path.resolve(__dirname, "./swc-decorators-transformer.cjs");
  }

  return path.resolve(__dirname, hasJsx ? "./swc-react-transformer.cjs" : "./swc-transformer.cjs");
}

/**
 * Jest configuration every TypeScript member of the workspace starts from.
 *
 * @param {{ rootDir: string, hasDecorators?: boolean, hasJsx?: boolean } & import("jest").Config} options - The
 *   member's root, whether its source uses TypeScript's legacy decorators or holds TSX components, and its overrides.
 * @returns {import("jest").Config} The member's configuration.
 */
function createJestConfig({ rootDir, hasDecorators = false, hasJsx = false, ...overrides }) {
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
      "^.+\\.[jt]sx?$": getTransformer(hasDecorators, hasJsx),
    },
    verbose: true,
    workerIdleMemoryLimit: "512MB",
    ...overrides,
  };
}

module.exports = { createJestConfig };
