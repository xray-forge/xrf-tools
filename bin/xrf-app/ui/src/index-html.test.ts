import { readFileSync } from "fs";
import { resolve } from "path";

import { describe, expect, it } from "@jest/globals";
import { createApplicationTheme, getSchemeVariables, getThemeVariables } from "@xrf/ui/theme/root";

describe("index.html", () => {
  it("reads only the theme properties the theme publishes", () => {
    const published: Set<string> = new Set([
      ...Object.keys(getThemeVariables(createApplicationTheme())),
      ...Object.keys(getSchemeVariables("dark")),
    ]);
    const markup: string = readFileSync(resolve(__dirname, "index.html"), "utf8");

    // The first paint draws the frame from these before any stylesheet loads; an unpublished name draws nothing.
    for (const [, name] of markup.matchAll(/var\((--xrf-[a-z0-9-]+)\)/g)) {
      expect({ name, isPublished: published.has(name) }).toEqual({ name, isPublished: true });
    }
  });
});
