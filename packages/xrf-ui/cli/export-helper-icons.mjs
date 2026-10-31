// Writes the Material icons the renderer's helper icon manifest names, as `@mui/icons-material` draws them, one SVG
// beside the manifest each, for the renderer's atlas generator to rasterise:
//
//   node cli/export-helper-icons.mjs <manifest.json>
//
// The manifest (`crates/xrf-renderer/helper-icons/manifest.json`) lists each icon by the name the renderer gives it,
// the Material icon it is drawn from and the file its picture is kept in, beside the manifest.

import { readFile, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";

import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

const manifestPath = process.argv[2];

if (!manifestPath) {
  console.error("Usage: node cli/export-helper-icons.mjs <manifest.json>");
  process.exit(1);
}

const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
const out = dirname(resolve(manifestPath));

for (const { icon, material, picture } of manifest.icons) {
  const { default: Icon } = await import(`@mui/icons-material/${material}`);
  const markup = renderToStaticMarkup(createElement(Icon));
  // The drawing inside the icon's own `svg`, whatever styling the component lays round it.
  const inner = markup.match(/<svg[^>]*>([\s\S]*)<\/svg>/)?.[1];

  if (!inner) {
    throw new Error(`No drawing found in the Material icon '${material}' for '${icon}'`);
  }

  await writeFile(
    join(out, picture),
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">${inner}</svg>\n`,
    "utf8"
  );
  console.log(`${icon} <- ${material}`);
}
