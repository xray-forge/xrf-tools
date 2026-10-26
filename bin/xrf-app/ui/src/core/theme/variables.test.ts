import { readFileSync } from "fs";
import { resolve } from "path";

import { describe, expect, it } from "@jest/globals";
import { Theme } from "@mui/material";

import { createApplicationTheme } from "./creation";
import { getSchemeVariables, getThemeVariables } from "./variables";

/** The stylesheets and markup that read the published properties, which is the whole other half of the bridge. */
const CONSUMERS: ReadonlyArray<string> = ["./tailwind.css", "../../index.html"];

/** Every `--xrf-*` name the application publishes, whether it is scheme-dependent or not. */
function readPublished(): Array<string> {
  const theme: Theme = createApplicationTheme();

  return [
    ...Object.keys(getThemeVariables(theme)),
    ...Object.keys(getSchemeVariables("dark")),
    ...Object.keys(getSchemeVariables("light")),
  ];
}

/** Every `--xrf-*` name the consumers read. */
function readConsumed(): Array<string> {
  const reference: RegExp = /var\((--xrf-[a-z0-9-]+)\)/g;

  return CONSUMERS.flatMap((it: string) => {
    const text: string = readFileSync(resolve(__dirname, it), "utf8");

    return [...text.matchAll(reference)].map(([, name]: RegExpMatchArray) => name);
  });
}

describe("theme variables", () => {
  it("publishes every property the stylesheets and markup read", () => {
    const published: Set<string> = new Set(readPublished());

    // A name read but never published resolves to nothing, which is a silent blank rather than a build failure.
    for (const name of new Set(readConsumed())) {
      expect({ name, isPublished: published.has(name) }).toEqual({ name, isPublished: true });
    }
  });

  it("publishes nothing the stylesheets and markup do not read", () => {
    const consumed: Set<string> = new Set(readConsumed());

    // A name published but never read is dead weight in every document the preload sheet lands in.
    for (const name of new Set(readPublished())) {
      expect({ name, isRead: consumed.has(name) }).toEqual({ name, isRead: true });
    }
  });

  it("states every scheme-dependent property in both schemes", () => {
    expect(Object.keys(getSchemeVariables("light"))).toEqual(Object.keys(getSchemeVariables("dark")));

    for (const [name, value] of Object.entries(getSchemeVariables("light"))) {
      // A value that matches its counterpart is a token that was only stated for one scheme.
      expect({
        name,
        differs: value !== getSchemeVariables("dark")[name as keyof ReturnType<typeof getSchemeVariables>],
      }).toEqual({ name, differs: true });
    }
  });
});
