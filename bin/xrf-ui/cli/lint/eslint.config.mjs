import * as path from "node:path";

import { fixupConfigRules } from "@eslint/compat";
import { createEslintConfig } from "@xrf/toolchain/eslint";
import reactPlugin from "eslint-plugin-react";
import reactHooksPlugin from "eslint-plugin-react-hooks";
import tailwindPlugin from "eslint-plugin-tailwindcss";

const RESTRICTED_IMPORT_PATHS = [
  {
    name: "@testing-library/react",
    importNames: ["screen"],
    message:
      "Query through the render result instead: `const { findByRole } = renderWithProviders(...)`. " +
      "`screen` searches the whole document, so a test cannot say which render it is asserting on.",
  },
];

/**
 * Flat configuration for the desktop frontend: the workspace rules, plus what only a React and Tailwind
 * application needs.
 *
 * Mirrors the layout of the engine repository so both codebases are linted the same way, but keeps this
 * package's own rule set: the shared part is the mechanism, not the policy.
 */
export default createEslintConfig(
  path.resolve(import.meta.dirname, "..", ".."),
  ...fixupConfigRules(reactPlugin.configs.flat.recommended),
  {
    files: ["**/*.ts", "**/*.tsx", "**/*.js", "**/*.mjs", "**/*.cjs"],
    settings: {
      react: { version: "detect" },
    },
    plugins: {
      "react-hooks": reactHooksPlugin,
    },
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: RESTRICTED_IMPORT_PATHS,
        },
      ],
      "react/jsx-curly-brace-presence": [
        "error",
        {
          props: "always",
          children: "never",
          propElementValues: "always",
        },
      ],
      "react/jsx-key": ["error", { checkFragmentShorthand: true }],
      "react/jsx-no-undef": "off",
      "react/no-unknown-property": "off",
      "react/prop-types": "off",
      "react/react-in-jsx-scope": "off",
      "react-hooks/exhaustive-deps": ["warn", { additionalHooks: "^useEditorPanels$" }],
      "react-hooks/rules-of-hooks": "error",
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
          paths: RESTRICTED_IMPORT_PATHS,
          patterns: [
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
    // A name absent from `@theme` generates no CSS and fails silently, which is the one way Tailwind can
    // break a screen without anything reporting it.
    files: ["src/**/*.tsx"],
    plugins: { tailwindcss: tailwindPlugin },
    settings: { tailwindcss: { cssConfigPath: "src/core/theme/tailwind.css" } },
    rules: {
      "tailwindcss/classnames-order": "warn",
      "tailwindcss/no-contradicting-classname": "error",
      // Real classes the application declares itself: `monospace` in the theme, and two markers a
      // parent selects on rather than styles directly.
      "tailwindcss/no-custom-classname": ["error", { whitelist: ["monospace", "workspace"] }],
    },
  },
  {
    // A test names a class to prove the prop reaches the DOM; the name is the assertion, not styling.
    files: ["src/**/*.test.tsx"],
    rules: {
      "tailwindcss/no-custom-classname": "off",
    },
  },
  {
    // Chrome colour belongs to the theme, where it is contrast-tested against every surface it can land on.
    files: ["src/**/*.{ts,tsx}"],
    ignores: [
      "src/core/theme/**",
      "**/*.test.{ts,tsx}",
      "src/core/brand/XrfMark.tsx",
      "src/core/shell/title-bar/WindowControlButton.tsx",
      "src/core/ui/media/media.styles.ts",
      "src/core/ui/media/ImageViewport/ImageViewport.tsx",
      "src/lib/logging/**",
    ],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector: String.raw`Literal[value=/#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})\b/]`,
          message:
            "Raw colour outside `core/theme`. Use a palette path or a theme token, so the value stays inside the " +
            "contrast rules the tokens define.",
        },
        {
          selector: String.raw`TemplateElement[value.raw=/#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})\b/]`,
          message:
            "Raw colour outside `core/theme`. Use a palette path or a theme token, so the value stays inside the " +
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
          paths: RESTRICTED_IMPORT_PATHS,
          patterns: [
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
