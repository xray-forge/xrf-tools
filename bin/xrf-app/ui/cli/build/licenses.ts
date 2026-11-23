import * as fs from "node:fs";
import * as path from "node:path";

import { type Plugin } from "vite";

/** One npm package the bundle includes, with the licence files it ships: the shape `cli/package/licenses.mjs` reads. */
interface IBundledPackage {
  name: string;
  version: string;
  license: string;
  files: Array<{ name: string; text: string }>;
}

const LICENCE_FILE: RegExp = /^(licen[cs]e|copying|notice)/i;

/**
 * @param id - A module id as the bundler reports it.
 * @returns The directory of the npm package the module belongs to, or null for a first-party or virtual module.
 */
function findPackageDirectory(id: string): string | null {
  const file: string = id.split("?")[0].replaceAll("\\", "/");
  const index: number = file.lastIndexOf("/node_modules/");

  if (file.startsWith("\0") || index < 0) {
    return null;
  }

  const [scope, name] = file.slice(index + "/node_modules/".length).split("/");

  return `${file.slice(0, index)}/node_modules/${scope.startsWith("@") ? `${scope}/${name}` : scope}`;
}

/**
 * @param directory - An npm package's directory.
 * @returns The package as the notices list it.
 * @throws {Error} When the package ships no licence file, whose text the notices cannot then carry.
 */
function readPackage(directory: string): IBundledPackage {
  const manifest: { name: string; version: string; license?: string } = JSON.parse(
    fs.readFileSync(path.join(directory, "package.json"), "utf8")
  );
  const files: Array<{ name: string; text: string }> = fs
    .readdirSync(directory)
    .filter((name: string) => LICENCE_FILE.test(name) && fs.statSync(path.join(directory, name)).isFile())
    .sort()
    .map((name: string) => ({ name, text: fs.readFileSync(path.join(directory, name), "utf8") }));

  if (files.length === 0) {
    throw new Error(`${manifest.name} ${manifest.version} ships no licence file in '${directory}'`);
  }

  return { name: manifest.name, version: manifest.version, license: manifest.license ?? "UNKNOWN", files };
}

/**
 * Writes every npm package the release bundle includes, with its licence texts, to `output` for the application's
 * archive (`cli/package/app.mjs` in the repository root). It reads what the bundler actually pulled in rather than the
 * manifest, since a transform can import a package the manifest lists only for development.
 *
 * The file sits outside the bundle's directory, which Tauri embeds into the executable.
 *
 * @param output - The JSON file to write.
 * @returns The plugin.
 */
// todo: Ship the licence of the Roboto font `public/fonts/roboto/` embeds: it is not a module, so this plugin never
//  sees it, and the committed files name neither their release nor their licence.
export function npmLicensesPlugin(output: string): Plugin {
  return {
    name: "xrf-npm-licenses",
    apply: "build",
    generateBundle() {
      const directories: Set<string> = new Set();

      for (const id of this.getModuleIds()) {
        const directory: string | null = findPackageDirectory(id);

        if (directory) {
          directories.add(directory);
        }
      }

      const packages: Array<IBundledPackage> = [...directories]
        .map(readPackage)
        .sort((left, right) => left.name.localeCompare(right.name) || left.version.localeCompare(right.version));

      fs.mkdirSync(path.dirname(output), { recursive: true });
      fs.writeFileSync(output, JSON.stringify(packages, null, 2));
    },
  };
}
