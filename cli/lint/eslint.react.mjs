import { fixupConfigRules } from "@eslint/compat";
import reactPlugin from "eslint-plugin-react";
import reactHooksPlugin from "eslint-plugin-react-hooks";
import tailwindPlugin from "eslint-plugin-tailwindcss";

/**
 * Imports a React member's tests may not use, for its own `no-restricted-imports` paths: a later block setting that
 * rule replaces an earlier one whole, so each member lists them where it lists its own.
 */
export const TESTING_LIBRARY_RESTRICTED_IMPORTS = [
  {
    name: "@testing-library/react",
    importNames: ["screen"],
    message:
      "Query through the render result instead: `const { findByRole } = renderWithProviders(...)`. " +
      "`screen` searches the whole document, so a test cannot say which render it is asserting on.",
  },
];

/**
 * What every React and Tailwind member of the workspace adds to the shared rules.
 *
 * @param {{ tailwindStylesheet: string, customClassNames?: Array<string>, effectHooks?: string }} options - The
 *   stylesheet declaring the Tailwind theme, the plain class names the member declares itself, and a pattern naming
 *   its own hooks that take dependency lists.
 * @returns {Array<import("eslint").Linter.Config>} The layers, to pass to `createEslintConfig`.
 */
export function createReactLayers({ tailwindStylesheet, customClassNames = [], effectHooks }) {
  return [
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
        "react-hooks/exhaustive-deps": ["warn", effectHooks ? { additionalHooks: effectHooks } : {}],
        "react-hooks/rules-of-hooks": "error",
      },
    },
    {
      // A name absent from `@theme` generates no CSS and fails silently, which is the one way Tailwind can
      // break a screen without anything reporting it.
      files: ["src/**/*.tsx"],
      plugins: { tailwindcss: tailwindPlugin },
      settings: { tailwindcss: { cssConfigPath: tailwindStylesheet } },
      rules: {
        "tailwindcss/classnames-order": "warn",
        "tailwindcss/no-contradicting-classname": "error",
        "tailwindcss/no-custom-classname": ["error", { whitelist: customClassNames }],
      },
    },
    {
      // A test names a class to prove the prop reaches the DOM; the name is the assertion, not styling.
      files: ["src/**/*.test.tsx"],
      rules: {
        "tailwindcss/no-custom-classname": "off",
      },
    },
  ];
}
