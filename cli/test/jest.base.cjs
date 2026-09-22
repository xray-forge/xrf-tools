const path = require("node:path");

/**
 * Jest configuration every TypeScript member of the workspace starts from.
 *
 * @param {{ rootDir: string } & import("jest").Config} options - The member's root, and its overrides.
 * @returns {import("jest").Config} The member's configuration.
 */
function createJestConfig({ rootDir, moduleNameMapper = {}, ...overrides }) {
  return {
    cacheDirectory: "<rootDir>/target/jest-cache",
    clearMocks: true,
    coverageDirectory: "<rootDir>/target/coverage-report",
    coveragePathIgnorePatterns: ["/node_modules/"],
    maxWorkers: "50%",
    moduleNameMapper: {
      // Exercise the same Three.js entries as Vite; r186 deprecates its CommonJS entry.
      // A member that imports three declares it, so its own `node_modules` holds the link.
      "^three$": "<rootDir>/node_modules/three/build/three.module.js",
      ...moduleNameMapper,
    },
    rootDir,
    roots: ["<rootDir>"],
    testEnvironment: "node",
    testMatch: ["<rootDir>/src/**/*.test.{ts,tsx}"],
    transform: {
      "^.+\\.[jt]sx?$": path.resolve(__dirname, "./swc-transformer.cjs"),
    },
    // Compile Three.js and its ESM addons together; leave other dependency runtimes intact.
    transformIgnorePatterns: ["^(?!.*/three/).*/node_modules/"],
    verbose: true,
    workerIdleMemoryLimit: "512MB",
    ...overrides,
  };
}

module.exports = { createJestConfig };
