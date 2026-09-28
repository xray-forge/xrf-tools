import { createEslintConfig } from "@xrf/toolchain/eslint";

/**
 * The renderer is linted with the workspace rules and nothing else: it holds no React and no application policy.
 */
export default createEslintConfig(import.meta.dirname);
