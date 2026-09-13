import { describe, expect, it } from "@jest/globals";

import { EStorageNamespace, isInStorageNamespace } from "@/core/storage/namespace";

describe("storage namespace", () => {
  it("carries no separator in a name, which is where one would go unnoticed", () => {
    for (const namespace of Object.values(EStorageNamespace)) {
      expect(namespace).toMatch(/^[a-z0-9]+(-[a-z0-9]+)*$/);
    }
  });

  it("claims only what is nested under it, not what merely starts alike", () => {
    expect(isInStorageNamespace("xrf.form.a.b", EStorageNamespace.FORM)).toBe(true);
    expect(isInStorageNamespace("xrf.form-recents.a.b", EStorageNamespace.FORM)).toBe(false);
    expect(isInStorageNamespace("xrf.formatting.a", EStorageNamespace.FORM)).toBe(false);
    expect(isInStorageNamespace("xrf.form", EStorageNamespace.FORM)).toBe(false);
  });
});
