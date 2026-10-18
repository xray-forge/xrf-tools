import { describe, expect, it } from "@jest/globals";

import { resolveDirectoryDestination } from "./resolve-directory-destination";

describe("resolveDirectoryDestination", () => {
  it("writes a nested directory as its own folder, without its parents", async () => {
    expect(await resolveDirectoryDestination("C:\\out", "configs\\gameplay")).toBe("C:\\out\\gameplay");
  });

  it("writes a top-level directory as its own folder", async () => {
    expect(await resolveDirectoryDestination("C:\\out", "scripts")).toBe("C:\\out\\scripts");
  });

  it("ignores a trailing separator and either slash style", async () => {
    expect(await resolveDirectoryDestination("C:\\out", "configs/gameplay\\")).toBe("C:\\out\\gameplay");
  });

  it("writes the tree root into the picked folder itself", async () => {
    expect(await resolveDirectoryDestination("C:\\out", "")).toBe("C:\\out");
  });
});
