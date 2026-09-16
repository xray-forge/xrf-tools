import { describe, expect, it } from "@jest/globals";

import { cn } from "@/lib/dom/dom-name";

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

  it("keeps classes that do not compete", () => {
    expect(cn("min-h-0 grow", "overflow-y-auto")).toBe("min-h-0 grow overflow-y-auto");
  });
});
