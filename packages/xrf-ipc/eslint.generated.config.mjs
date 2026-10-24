import baseConfig from "./eslint.config.mjs";

/**
 * Flat configuration for the generated Rust mirrors, used only by `format:bindings`.
 */
export default [
  ...baseConfig,
  {
    files: ["src/commands/**/*.ts", "src/types/**/*.ts"],
    rules: {
      "max-len": "off",
    },
  },
];
