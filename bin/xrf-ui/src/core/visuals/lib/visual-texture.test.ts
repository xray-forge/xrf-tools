import { describe, expect, it } from "@jest/globals";

import { EVisualTextureState, toInitialTextureState } from "@/core/visuals/lib/visual-texture";
import { mockTextureDependency } from "@/fixtures/mocks/visual.mocks";

describe("toInitialTextureState", () => {
  it("separates a texture that was not found from a reference that was never usable", () => {
    // Both end up untextured, and only one is the model's fault, so the panel must not report them the same way.
    expect(toInitialTextureState({ kind: "noScope" })).toBe(EVisualTextureState.UNRESOLVED);
    expect(toInitialTextureState({ kind: "missing", roots: ["C:\\gamedata"] })).toBe(EVisualTextureState.UNRESOLVED);
    expect(toInitialTextureState({ kind: "rejected", reason: "not a logical path" })).toBe(EVisualTextureState.FAILED);
    expect(toInitialTextureState(mockTextureDependency().resolution)).toBe(EVisualTextureState.LOADING);
  });
});
