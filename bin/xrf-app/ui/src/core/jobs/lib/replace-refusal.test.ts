import { describe, expect, it } from "@jest/globals";

import { isReplaceRefusal } from "./replace-refusal";

describe("isReplaceRefusal", () => {
  it("tells a refusal over files already there from any other failure", () => {
    expect(
      isReplaceRefusal(new Error("C:\\out already holds what this run writes (all.spawn); force it to replace them"))
    ).toBe(true);
    expect(isReplaceRefusal(new Error("The spawn file is not one"))).toBe(false);
    expect(isReplaceRefusal(null)).toBe(false);
  });
});
