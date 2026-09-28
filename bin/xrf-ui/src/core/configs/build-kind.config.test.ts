import { describe, expect, it } from "@jest/globals";

import { getBuildKind } from "@/core/configs/build-kind.config";
import { EBuildKind } from "@/core/ipc/types/xrf-build-info";
import { setMockBuildKind } from "@/fixtures/mocks/build-kind.mocks";

describe("getBuildKind", () => {
  it("reads the kind the binary recorded", () => {
    for (const kind of Object.values(EBuildKind)) {
      setMockBuildKind(kind);

      expect(getBuildKind()).toBe(kind);
    }
  });

  it("reads a document no binary hosts as local", () => {
    expect(getBuildKind()).toBe(EBuildKind.LOCAL);
  });

  it("reads a kind it does not know as local, as the binary itself does", () => {
    setMockBuildKind("nightly");

    expect(getBuildKind()).toBe(EBuildKind.LOCAL);
  });
});
