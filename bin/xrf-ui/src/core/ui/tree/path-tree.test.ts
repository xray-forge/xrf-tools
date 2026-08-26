import { describe, expect, it } from "@jest/globals";

import { splitLogicalPath } from "@/core/ui/tree/path-tree";

describe("splitLogicalPath", () => {
  it("separates the name a reader identifies a file by from the directories that place it", () => {
    expect(splitLogicalPath("meshes\\actors\\stalker_bandit\\stalker_bandit_1.ogf")).toEqual({
      directory: "meshes\\actors\\stalker_bandit",
      name: "stalker_bandit_1.ogf",
    });
  });

  it("reports no directory for a file at the root, so a caller can leave the line out", () => {
    expect(splitLogicalPath("particles.xr")).toEqual({ directory: null, name: "particles.xr" });
  });

  it("keeps every separator but the last, which is what makes the directory a path", () => {
    expect(splitLogicalPath("a\\b\\c")).toEqual({ directory: "a\\b", name: "c" });
  });

  it("splits an empty path into nothing rather than failing", () => {
    expect(splitLogicalPath("")).toEqual({ directory: null, name: "" });
  });
});
