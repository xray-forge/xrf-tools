import jsPlugin from "@eslint/js";
import importPlugin from "eslint-plugin-import";
import jestPlugin from "eslint-plugin-jest";
import jsdocPlugin from "eslint-plugin-jsdoc";
import reactPlugin from "eslint-plugin-react";
import reactHooksPlugin from "eslint-plugin-react-hooks";
import sortKeysFixPlugin from "eslint-plugin-sort-keys-fix";
import unusedImportsPlugin from "eslint-plugin-unused-imports";
import globals from "globals";
import tsPlugin from "typescript-eslint";

/**
 * Flat configuration for the desktop frontend.
 *
 * Mirrors the layout of the engine repository so both codebases are linted the same way, but keeps this
 * package's own rule set: the shared part is the mechanism, not the policy.
 */
export default [
  {
    ignores: ["target/**", "dist/**"],
  },
  jsPlugin.configs.recommended,
  ...tsPlugin.configs.recommended,
  importPlugin.flatConfigs.recommended,
  importPlugin.flatConfigs.typescript,
  jestPlugin.configs["flat/style"],
  jsdocPlugin.configs["flat/recommended"],
  reactPlugin.configs.flat.recommended,
  {
    files: ["**/*.ts", "**/*.tsx", "**/*.js", "**/*.mjs", "**/*.cjs"],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    settings: {
      react: { version: "18" },
    },
    plugins: {
      "react-hooks": reactHooksPlugin,
      "sort-keys-fix": sortKeysFixPlugin,
      "unused-imports": unusedImportsPlugin,
    },
    rules: {
      "@typescript-eslint/no-misused-new": "off",
      "@typescript-eslint/array-type": ["error", { default: "generic" }],
      "@typescript-eslint/ban-ts-comment": "off",
      "@typescript-eslint/explicit-member-accessibility": ["error"],
      "@typescript-eslint/explicit-module-boundary-types": "off",
      "@typescript-eslint/no-empty-function": "off",
      // Replaces `no-empty-interface`, which this rule absorbed in typescript-eslint v8.
      "@typescript-eslint/no-empty-object-type": "off",
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-inferrable-types": "off",
      "@typescript-eslint/no-non-null-assertion": "off",
      // The jest and vite tooling in `cli/` is CommonJS by necessity.
      "@typescript-eslint/no-require-imports": "off",
      // Turned off in favour of the `unused-imports` variant, which can also strip the import.
      "@typescript-eslint/no-unused-vars": "off",
      "unused-imports/no-unused-imports": "error",
      "unused-imports/no-unused-vars": [
        "error",
        {
          vars: "all",
          varsIgnorePattern: "^_",
          args: "none",
          reportUsedIgnorePattern: true,
        },
      ],
      "jsdoc/check-tag-names": "off",
      "jsdoc/tag-lines": [
        "error",
        "any",
        {
          startLines: 1,
          endLines: 0,
        },
      ],
      "jsdoc/require-jsdoc": "off",
      "jsdoc/require-param": "off",
      "jsdoc/require-param-description": "off",
      "jsdoc/require-param-type": "off",
      "jsdoc/require-returns": "off",
      "jsdoc/require-returns-type": "off",
      "jsdoc/require-yields": "off",
      "jsdoc/require-yields-type": "off",
      "array-element-newline": ["error", "consistent"],
      "arrow-parens": ["error", "always"],
      "arrow-spacing": "error",
      "brace-style": "error",
      camelcase: "off",
      "func-style": ["error", "declaration"],
      "no-inner-declarations": "off",
      "prefer-arrow-callback": "error",
      "sort-vars": "error",
      "sort-imports": [
        "error",
        {
          ignoreCase: true,
          ignoreDeclarationSort: true,
          ignoreMemberSort: false,
          allowSeparatedGroups: false,
        },
      ],
      "comma-dangle": [
        "error",
        {
          arrays: "always-multiline",
          objects: "always-multiline",
          imports: "always-multiline",
          exports: "never",
          functions: "never",
        },
      ],
      eqeqeq: "error",
      "eol-last": ["error", "always"],
      "func-call-spacing": ["error", "never"],
      "function-paren-newline": "off",
      "import/default": "off",
      "import/no-relative-parent-imports": "error",
      "import/no-unresolved": "off",
      "import/order": [
        "error",
        {
          alphabetize: { caseInsensitive: true, order: "asc" },
          // Relative specifiers share one rank, so they form a single block below everything else
          // rather than three blocks separated by blank lines.
          groups: ["builtin", "external", ["parent", "sibling", "index"]],
          "newlines-between": "always",
          pathGroups: [{ group: "external", pattern: "@/**", position: "after" }],
          pathGroupsExcludedImportTypes: ["builtin"],
        },
      ],
      "key-spacing": ["error", { afterColon: true, beforeColon: false }],
      "keyword-spacing": "error",
      "linebreak-style": ["error", "unix"],
      "max-len": ["error", { code: 120, ignorePattern: "^import\\W.*" }],
      "no-constructor-return": "error",
      "no-duplicate-imports": "error",
      "no-multi-spaces": "error",
      "no-multiple-empty-lines": ["error", { max: 1, maxBOF: 1 }],
      "no-trailing-spaces": "error",
      "object-curly-newline": ["error", { consistent: true, multiline: true }],
      "object-curly-spacing": ["error", "always"],
      "object-property-newline": ["error", { allowAllPropertiesOnSameLine: true }],
      "padding-line-between-statements": [
        "error",
        { blankLine: "always", next: "return", prev: "*" },
        { blankLine: "always", next: ["const", "let", "var"], prev: "expression" },
        { blankLine: "always", next: "*", prev: ["const", "let", "var"] },
        { blankLine: "always", next: "*", prev: ["for", "if", "while", "do", "with"] },
        { blankLine: "always", next: ["function", "class"], prev: ["function", "class"] },
        { blankLine: "any", next: ["const", "let", "var"], prev: ["const", "let", "var"] },
      ],
      "react/jsx-curly-brace-presence": [
        "error",
        {
          props: "always",
          children: "never",
          propElementValues: "always",
        },
      ],
      "react/jsx-key": "off",
      "react/jsx-no-undef": "off",
      "react/no-unknown-property": "off",
      "react/prop-types": "off",
      "react/react-in-jsx-scope": "off",
      "react-hooks/exhaustive-deps": ["warn", { additionalHooks: "^useEditorPanels$" }],
      semi: "error",
      "space-in-parens": ["error", "never"],
      "spaced-comment": ["error", "always"],
      "template-tag-spacing": ["error", "never"],
      yoda: "error",
    },
  },
  {
    // Accessibility modifiers are TypeScript syntax, so the rule cannot be satisfied in plain JavaScript.
    files: ["**/*.js", "**/*.mjs", "**/*.cjs"],
    rules: {
      "@typescript-eslint/explicit-member-accessibility": "off",
    },
  },
  {
    files: ["**/*.d.ts"],
    rules: {
      "unused-imports/no-unused-vars": "off",
    },
  },
];
