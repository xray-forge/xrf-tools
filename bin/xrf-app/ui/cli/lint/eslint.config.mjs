import * as path from "node:path";
import { fileURLToPath } from "node:url";

import { createEslintConfig } from "@xrf/toolchain/eslint";
import { createReactLayers, TESTING_LIBRARY_RESTRICTED_IMPORTS } from "@xrf/toolchain/eslint-react";

/**
 * MUI and Emotion reach the application only through `@xrf/ui`, which owns the look they draw.
 */
const KIT_RESTRICTED_IMPORT_PATTERNS = [
  {
    group: ["@mui/**", "@emotion/**"],
    message:
      "MUI and Emotion are `@xrf/ui`'s to import. Take the name from its subpath (`@xrf/ui/material`, `/icons`, " +
      "`/data-grid`, `/virtualizer`), adding it there if it is missing.",
  },
];

/**
 * Flat configuration for the desktop frontend: the workspace rules and the React and Tailwind layers it shares with
 * `@xrf/ui`, plus the boundaries only this application keeps: core and libraries apart from applications, MUI and
 * Emotion behind the kit, and colour behind the theme.
 */
export default createEslintConfig(
  path.resolve(import.meta.dirname, "..", ".."),
  ...createReactLayers({
    tailwindStylesheet: fileURLToPath(import.meta.resolve("@xrf/ui/theme/tailwind.css")),
    // `monospace` is declared by the theme; the other two are markers a parent selects on rather than styles directly.
    customClassNames: ["monospace", "workspace"],
    effectHooks: "^useEditor(Panels|Menus)$",
  }),
  {
    files: ["**/*.ts", "**/*.tsx", "**/*.js", "**/*.mjs", "**/*.cjs"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: TESTING_LIBRARY_RESTRICTED_IMPORTS,
          patterns: KIT_RESTRICTED_IMPORT_PATTERNS,
        },
      ],
    },
  },
  {
    files: ["src/core/**/*.{ts,tsx}"],
    // Integration tests may compose shared components with application services.
    ignores: ["**/*.test.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: TESTING_LIBRARY_RESTRICTED_IMPORTS,
          patterns: [
            ...KIT_RESTRICTED_IMPORT_PATTERNS,
            {
              group: ["@/applications", "@/applications/**"],
              message: "Core must not depend on applications. Inject application behavior at the composition root.",
            },
          ],
        },
      ],
    },
  },
  {
    // Chrome colour belongs to the theme in `@xrf/ui`, where it is contrast-tested against every surface it can land on.
    files: ["src/**/*.{ts,tsx}"],
    ignores: [
      "**/*.test.{ts,tsx}",
      "src/core/brand/XrfMark.tsx",
      "src/core/shell/title-bar/WindowControlButton.tsx",
      "src/core/ui/media/media.styles.ts",
      "src/core/ui/media/ImageViewport/ImageViewport.tsx",
    ],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector: String.raw`Literal[value=/#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})\b/]`,
          message:
            "Raw colour outside the theme. Use a palette path or a theme token, so the value stays inside the " +
            "contrast rules the tokens define.",
        },
        {
          selector: String.raw`TemplateElement[value.raw=/#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})\b/]`,
          message:
            "Raw colour outside the theme. Use a palette path or a theme token, so the value stays inside the " +
            "contrast rules the tokens define.",
        },
      ],
    },
  },
  {
    files: ["src/lib/**/*.{ts,tsx}"],
    ignores: ["**/*.test.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: TESTING_LIBRARY_RESTRICTED_IMPORTS,
          patterns: [
            ...KIT_RESTRICTED_IMPORT_PATTERNS,
            {
              group: ["@/core", "@/core/**", "@/applications", "@/applications/**"],
              message: "Library utilities must not depend on core or applications. Move domain behavior to its owner.",
            },
          ],
        },
      ],
    },
  }
);
