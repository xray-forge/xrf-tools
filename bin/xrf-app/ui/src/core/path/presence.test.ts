import { describe, expect, it, jest } from "@jest/globals";
import { exists } from "@tauri-apps/plugin-fs";

import { isPathPresent } from "@/core/path/presence";

describe("isPathPresent", () => {
  it("answers what the host says of a path it looks at", async () => {
    jest.mocked(exists).mockResolvedValueOnce(true).mockResolvedValueOnce(false);

    expect(await isPathPresent("C:\\game")).toBe(true);
    expect(await isPathPresent("C:\\gone")).toBe(false);
  });

  it("reads a look the host refuses as absent rather than failing", async () => {
    jest.mocked(exists).mockRejectedValueOnce(new Error("forbidden path"));

    expect(await isPathPresent("C:\\denied")).toBe(false);
  });
});
