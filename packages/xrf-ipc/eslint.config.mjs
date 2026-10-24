import { createEslintConfig } from "@xrf/toolchain/eslint";

/** What a module of the root layer may not reach: the frameworks and the layers built on it. */
const ABOVE_ROOT = [
  { group: ["@wirestate/*", "mobx", "react"], message: "The root layer is framework-free: move this to `session/`." },
  { group: ["#/session/*", "#/react/*"], message: "The root layer cannot import a layer built on it." },
];

/** What the session layer may not reach. */
const ABOVE_SESSION = [
  { group: ["@wirestate/react", "react"], message: "The session layer renders nothing: move this to `react/`." },
  { group: ["#/react/*"], message: "The session layer cannot import the React layer built on it." },
];

/** What no shipped module may reach. */
const TEST_ONLY = [{ group: ["#/testing/*", "@jest/*"], message: "Only tests and `testing/` import test helpers." }];

/**
 * The package's layers, each importing only those below it: the root (`invoke`, `raw`, `bulk/`, `metrics/`, the
 * generated `types/` and `commands/`), then `session/`, then `react/`; `testing/` sits beside them for tests alone.
 */
export default createEslintConfig(
  import.meta.dirname,
  {
    files: ["src/**/*.ts"],
    ignores: ["src/session/**", "src/react/**", "src/testing/**", "src/**/*.test.ts"],
    rules: {
      "no-restricted-imports": ["error", { patterns: [...ABOVE_ROOT, ...TEST_ONLY] }],
    },
  },
  {
    files: ["src/session/**/*.ts"],
    ignores: ["src/**/*.test.ts"],
    rules: {
      "no-restricted-imports": ["error", { patterns: [...ABOVE_SESSION, ...TEST_ONLY] }],
    },
  },
  {
    files: ["src/react/**/*.ts"],
    ignores: ["src/**/*.test.ts"],
    rules: {
      "no-restricted-imports": ["error", { patterns: TEST_ONLY }],
    },
  }
);
