import { describe, expect, it } from "@jest/globals";

import { SelectedVisualDescription } from "@/core/ipc/types/xrf-app";
import { ERenderTextureState, RenderTextureReport } from "@/core/ipc/types/xrf-renderer";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { toVisualBumpStatuses, toVisualTextureStatuses } from "@/core/visuals/lib/visual-texture-report";
import { mockMaterialDescriptor, mockSelectedVisual, mockTextureDependency } from "@/fixtures/mocks/visual.mocks";

function mockLoaded(reference: string, isExpanded: boolean = false): RenderTextureReport {
  return {
    reference,
    state: { height: 4, isExpanded, kind: ERenderTextureState.LOADED, layout: "DXT1", levels: 1, width: 4 },
  };
}

function mockVisual(): SelectedVisualDescription {
  return mockSelectedVisual({
    dependencies: {
      motions: [],
      textures: [
        mockTextureDependency({ submeshIndex: 0 }),
        mockTextureDependency({
          reference: "wpn\\wpn_lost",
          resolution: { kind: "missing", roots: ["C:\\gamedata"] },
          submeshIndex: 1,
        }),
        mockTextureDependency({
          reference: "wpn:bad",
          resolution: { kind: "rejected", reason: "not a logical path" },
          submeshIndex: 2,
        }),
      ],
    },
    materials: { "wpn\\wpn_ak74": mockMaterialDescriptor() },
  });
}

describe("toVisualTextureStatuses", () => {
  it("keeps a located texture loading until the renderer reports it", () => {
    const statuses = toVisualTextureStatuses(mockVisual(), []);

    expect([...statuses.values()].map((it) => it.state)).toEqual([
      EVisualTextureState.LOADING,
      EVisualTextureState.UNRESOLVED,
      EVisualTextureState.FAILED,
    ]);
  });

  it("takes what the renderer made of a located texture, matching its reference whatever its spelling", () => {
    const loaded = toVisualTextureStatuses(mockVisual(), [mockLoaded("WPN/wpn_ak74")]);
    const expanded = toVisualTextureStatuses(mockVisual(), [mockLoaded("wpn\\wpn_ak74", true)]);
    const failed = toVisualTextureStatuses(mockVisual(), [
      { reference: "wpn\\wpn_ak74", state: { kind: ERenderTextureState.FAILED, reason: "truncated" } },
    ]);

    expect(loaded.get(0)?.state).toBe(EVisualTextureState.APPLIED);
    expect(expanded.get(0)?.state).toBe(EVisualTextureState.DECODED);
    expect(failed.get(0)).toEqual({ reason: "truncated", state: EVisualTextureState.FAILED, submeshIndex: 0 });
  });
});

describe("toVisualBumpStatuses", () => {
  it("reports each half of a located pair on its own", () => {
    const statuses = toVisualBumpStatuses(mockVisual(), [
      mockLoaded("wpn\\wpn_ak74_bump"),
      { reference: "wpn\\wpn_ak74_bump#", state: { kind: ERenderTextureState.MISSING } },
    ]);

    expect([...statuses.values()]).toEqual([
      {
        bump: EVisualTextureState.APPLIED,
        companion: EVisualTextureState.UNRESOLVED,
        reason: null,
        submeshIndex: 0,
      },
    ]);
  });
});
