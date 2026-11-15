import { createEslintConfig } from "@xrf/toolchain/eslint";
import { createReactLayers, TESTING_LIBRARY_RESTRICTED_IMPORTS } from "@xrf/toolchain/eslint-react";

/**
 * The workspace rules, plus the React layers. Nothing here is styled, so no Tailwind rules apply.
 */
export default createEslintConfig(import.meta.dirname, ...createReactLayers(), {
  files: ["**/*.ts", "**/*.tsx"],
  rules: {
    "no-restricted-imports": ["error", { paths: TESTING_LIBRARY_RESTRICTED_IMPORTS }],
  },
});
