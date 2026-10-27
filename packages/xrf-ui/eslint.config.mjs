import { fileURLToPath } from "node:url";

import { createEslintConfig } from "@xrf/toolchain/eslint";
import { createReactLayers, TESTING_LIBRARY_RESTRICTED_IMPORTS } from "@xrf/toolchain/eslint-react";

/**
 * The workspace rules, plus the React and Tailwind layers. MUI and Emotion are this package's to import, and raw colour
 * is its to state, so neither carries the restriction the application lints under.
 */
export default createEslintConfig(
  import.meta.dirname,
  ...createReactLayers({
    tailwindStylesheet: fileURLToPath(new URL("./src/theme/root/tailwind.css", import.meta.url)),
    customClassNames: ["monospace"],
  }),
  {
    files: ["**/*.ts", "**/*.tsx"],
    rules: {
      "no-restricted-imports": ["error", { paths: TESTING_LIBRARY_RESTRICTED_IMPORTS }],
    },
  }
);
