import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

import { createEslintConfig, createImportCycleLayer } from "@xrf/toolchain/eslint";
import { createReactLayers, TESTING_LIBRARY_RESTRICTED_IMPORTS } from "@xrf/toolchain/eslint-react";

const ROOT_DIR = path.resolve(import.meta.dirname, "..", "..");

/** Every tool's directory name under `src/tools`, each a slug no other tool imports. */
const TOOL_SLUGS = fs
  .readdirSync(path.join(ROOT_DIR, "src", "tools"), { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name);

/**
 * MUI and Emotion reach the application only through `@xrf/ui`, which owns the look they draw.
 */
const KIT_RESTRICTED_IMPORT_PATTERNS = [
  {
    group: ["@mui/**", "@emotion/**"],
    message:
      "MUI and Emotion are `@xrf/ui`'s to import. Take the name from the family that passes it through " +
      "(`@xrf/ui/button`, `/input`, `/overlay`, `/icons/<name>`, `/data-grid`, `/virtualizer`, ...), adding it " +
      "there if it is missing.",
  },
];

/**
 * The native file dialogs reach the application only through `core/path/native-dialog`, which resolves where they open
 * and answers a plain path.
 */
const NATIVE_DIALOG_RESTRICTED_IMPORT_PATTERN = {
  group: ["@tauri-apps/plugin-dialog"],
  message:
    "Ask through `@/core/path/native-dialog` (`askToOpen`, `askToOpenDirectory`, `askToSave`), which opens where " +
    "the path held leads and answers a plain path.",
};

/** Files that reach the dialog plugin itself: its one wrapper, and the tests and fixtures that mock it. */
const NATIVE_DIALOG_OWNERS = ["src/core/path/native-dialog.ts", "**/*.test.{ts,tsx}", "src/fixtures/**"];

/**
 * One `no-restricted-imports` layer, with the native dialog ban for every file it covers but the dialog's owners, which
 * a second layer gives the same patterns without it. ESLint replaces a rule's options per matching layer rather than
 * merging them, so the exemption restates the layer's own patterns.
 *
 * @param layer - The layer.
 * @param layer.files - Files it covers.
 * @param layer.ignores - Files among them it leaves out.
 * @param layer.patterns - Imports it restricts.
 * @returns The layer for the dialog's users, then the one for its owners among the same files.
 */
function restrictImports({ files, ignores = [], patterns }) {
  return [
    {
      files,
      ignores: [...ignores, ...NATIVE_DIALOG_OWNERS],
      rules: toRestrictedImportsRule([...patterns, NATIVE_DIALOG_RESTRICTED_IMPORT_PATTERN]),
    },
    {
      // An inner list matches a file only when every glob in it does.
      files: files.flatMap((glob) => NATIVE_DIALOG_OWNERS.map((owner) => [glob, owner])),
      ignores,
      rules: toRestrictedImportsRule(patterns),
    },
  ];
}

/**
 * @param patterns - Imports to restrict, beside the testing library's paths.
 * @returns The rule entry.
 */
function toRestrictedImportsRule(patterns) {
  return {
    "no-restricted-imports": ["error", { paths: TESTING_LIBRARY_RESTRICTED_IMPORTS, patterns }],
  };
}

/**
 * Flat configuration for the desktop frontend: the workspace rules and the React and Tailwind layers it shares with
 * `@xrf/ui`, plus the boundaries only this application keeps: core and libraries apart from tools, tools apart from
 * each other, core free of import cycles, MUI and Emotion behind the kit, native dialogs behind one module, and colour
 * behind the theme.
 */
export default createEslintConfig(
  ROOT_DIR,
  ...createReactLayers({
    tailwindStylesheet: fileURLToPath(import.meta.resolve("@xrf/ui/theme/tailwind.css")),
    // `monospace` is declared by the theme; the other two are markers a parent selects on rather than styles directly.
    customClassNames: ["monospace", "workspace"],
    effectHooks: "^useShell(Panels|Menus)$",
  }),
  ...restrictImports({
    files: ["**/*.ts", "**/*.tsx", "**/*.js", "**/*.mjs", "**/*.cjs"],
    patterns: KIT_RESTRICTED_IMPORT_PATTERNS,
  }),
  ...restrictImports({
    files: ["src/core/**/*.{ts,tsx}"],
    // Integration tests may compose shared components with tool services.
    ignores: ["**/*.test.{ts,tsx}"],
    patterns: [
      ...KIT_RESTRICTED_IMPORT_PATTERNS,
      {
        group: ["@/tools", "@/tools/**"],
        message: "Core must not depend on tools. Inject tool behavior at the composition root.",
      },
    ],
  }),
  createImportCycleLayer({
    files: ["src/core/**/*.{ts,tsx}"],
    // Tests compose a domain with whatever it is tested against; only what ships has to load in one order.
    ignores: ["**/*.test.{ts,tsx}"],
    project: path.join(ROOT_DIR, "tsconfig.app.json"),
  }),
  ...TOOL_SLUGS.flatMap((slug) =>
    restrictImports({
      files: [`src/tools/${slug}/**/*.{ts,tsx}`],
      patterns: [
        ...KIT_RESTRICTED_IMPORT_PATTERNS,
        {
          group: ["@/tools/*", "@/tools/*/**", `!@/tools/${slug}`, `!@/tools/${slug}/**`],
          message:
            "A tool must not depend on another tool. Move what both need to `core/`, or register it at the " +
            "composition root.",
        },
      ],
    })
  ),
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
  ...restrictImports({
    files: ["src/lib/**/*.{ts,tsx}"],
    ignores: ["**/*.test.{ts,tsx}"],
    patterns: [
      ...KIT_RESTRICTED_IMPORT_PATTERNS,
      {
        group: ["@/core", "@/core/**", "@/tools", "@/tools/**"],
        message: "Library utilities must not depend on core or tools. Move domain behavior to its owner.",
      },
    ],
  })
);
