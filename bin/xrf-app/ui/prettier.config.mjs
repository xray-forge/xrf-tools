import { fileURLToPath } from "node:url";

import base from "@xrf/toolchain/prettier";

/**
 * The workspace formatting, plus the Tailwind class ordering only this application's sources need.
 *
 * Both paths resolve from this file, since the commit hook runs Prettier from the workspace root.
 *
 * @type {import("prettier").Config}
 */
export default {
  ...base,
  plugins: [fileURLToPath(import.meta.resolve("prettier-plugin-tailwindcss"))],
  tailwindStylesheet: fileURLToPath(import.meta.resolve("@xrf/ui/theme/tailwind.css")),
};
