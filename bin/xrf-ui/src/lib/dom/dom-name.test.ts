import { readFileSync } from "fs";
import { resolve } from "path";

import { describe, expect, it } from "@jest/globals";

import { cn } from "@/lib/dom/dom-name";

/**
 * Namespaces whose custom names a merger has to be told about; colours it matches generically.
 */
const MERGED_NAMESPACES: Record<string, { prefix: string; rival: string }> = {
  container: { prefix: "max-w-", rival: "max-w-prose" },
  font: { prefix: "font-", rival: "font-sans" },
  leading: { prefix: "leading-", rival: "leading-tight" },
  radius: { prefix: "rounded-", rival: "rounded-none" },
  spacing: { prefix: "h-", rival: "h-0" },
  text: { prefix: "text-", rival: "text-sm" },
};

/**
 * The custom names `tailwind.css` declares, so the merger's list cannot quietly fall behind the theme.
 */
function readThemeNames(namespace: string): Array<string> {
  const css: string = readFileSync(resolve(__dirname, "../../core/theme/tailwind.css"), "utf8");
  const declaration: RegExp = new RegExp(`^\\s*--${namespace}-([a-z0-9-]+):`, "gm");

  return [...css.matchAll(declaration)].map(([, name]: RegExpMatchArray) => name);
}

describe("cn", () => {
  it("joins what it is given and drops the falsy", () => {
    expect(cn("grow", false, undefined, "mx-auto", null)).toBe("grow mx-auto");
  });

  it("lets the later class win where two set the same property", () => {
    expect(cn("overflow-y-auto", "overflow-y-hidden")).toBe("overflow-y-hidden");
    expect(cn("mx-auto", "mx-0")).toBe("mx-0");
  });

  it("resolves the application's own scales, which the merger only knows because it is told", () => {
    expect(cn("max-w-reading", "max-w-prose")).toBe("max-w-prose");
    expect(cn("h-tree-row", "h-10")).toBe("h-10");
    expect(cn("w-tree-icon", "w-4")).toBe("w-4");
    expect(cn("gap-tree-gap", "gap-2")).toBe("gap-2");
    expect(cn("rounded-surface", "rounded-none")).toBe("rounded-none");
  });

  it("knows every custom scale the theme declares, so a later class can win", () => {
    for (const [namespace, { prefix, rival }] of Object.entries(MERGED_NAMESPACES)) {
      for (const name of readThemeNames(namespace)) {
        // A name the merger does not know leaves both classes in place, which is the silent failure.
        expect({ name, merged: cn(`${prefix}${name}`, rival) }).toEqual({ name, merged: rival });
      }
    }
  });

  it("keeps classes that do not compete", () => {
    expect(cn("min-h-0 grow", "overflow-y-auto")).toBe("min-h-0 grow overflow-y-auto");
  });
});
