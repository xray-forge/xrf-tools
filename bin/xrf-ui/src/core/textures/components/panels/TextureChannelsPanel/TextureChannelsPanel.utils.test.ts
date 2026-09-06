import { describe, expect, it } from "@jest/globals";
import { Texture } from "three";

import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import { EMPTY_TEXTURE_SURFACE, ITextureSurfaceTextures } from "@/core/textures/lib/texture-surface";
import { EVisualBumpView } from "@/core/visuals/lib/visual-bump-channels";
import { mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockMaterialDescriptor } from "@/fixtures/mocks/visual.mocks";

import {
  describeTextureChannelsGap,
  TEXTURE_CHANNEL_TILES,
  toTextureChannelAspect,
} from "./TextureChannelsPanel.utils";

/** A texture whose descriptor declares a pair with both halves located. */
function mockBumped(): TextureDescription {
  return mockTextureDescription(undefined, { material: mockMaterialDescriptor() });
}

/** The same, with one half nothing located. */
function mockHalfLocated(): TextureDescription {
  const material: XrayMaterialDescriptor = mockMaterialDescriptor();

  return mockTextureDescription(undefined, {
    material: {
      ...material,
      bump: material.bump && {
        ...material.bump,
        companion: {
          ...material.bump.companion,
          resolution: { kind: "missing", roots: ["C:\\gamedata"] },
        },
      },
    },
  });
}

/** An upload that produced both halves. */
function mockUploaded(): ITextureSurfaceTextures {
  return { aspect: 1, base: null, bump: { bump: new Texture(), companion: new Texture() } };
}

describe("TEXTURE_CHANNEL_TILES", () => {
  it("shows the two files before the three values read out of them", () => {
    // Reading a reconstruction before its inputs invites believing a decode that was fed the wrong plane.
    expect(TEXTURE_CHANNEL_TILES.map((it) => it.view)).toEqual([
      EVisualBumpView.BUMP,
      EVisualBumpView.COMPANION,
      EVisualBumpView.NORMAL,
      EVisualBumpView.GLOSS,
      EVisualBumpView.HEIGHT,
    ]);
  });
});

describe("toTextureChannelAspect", () => {
  it("takes the proportions of the pair, not of the panel", () => {
    // A 1:2 plane forced square shows every measurement of its detail stretched, which is what a person is judging.
    const textures: ITextureSurfaceTextures = mockUploaded();

    (textures.bump!.bump as unknown as { image: unknown }).image = { height: 2048, width: 1024 };

    expect(toTextureChannelAspect(textures)).toBe("1024 / 2048");
  });

  it("falls back to a square before a pair is bound", () => {
    expect(toTextureChannelAspect(null)).toBe("1 / 1");
    expect(toTextureChannelAspect(EMPTY_TEXTURE_SURFACE)).toBe("1 / 1");
  });
});

describe("describeTextureChannelsGap", () => {
  it("says nothing is selected when nothing is", () => {
    expect(describeTextureChannelsGap(null, null, false)).toMatch(/No texture selected/);
  });

  it("says a texture declaring no pair has no planes to read", () => {
    expect(describeTextureChannelsGap(mockTextureDescription(), EMPTY_TEXTURE_SURFACE, false)).toMatch(
      /declares no bump pair/
    );
  });

  it("draws the tiles once both halves are uploaded", () => {
    expect(describeTextureChannelsGap(mockBumped(), mockUploaded(), false)).toBeNull();
  });

  it("waits while the pair is still being read", () => {
    expect(describeTextureChannelsGap(mockBumped(), EMPTY_TEXTURE_SURFACE, true)).toMatch(/Reading/);
  });

  it("names the half that was not found", () => {
    // Which half is missing separates a typo in a descriptor from a file that was never shipped.
    const gap: string = describeTextureChannelsGap(mockHalfLocated(), EMPTY_TEXTURE_SURFACE, false) ?? "";

    expect(gap).toContain("wpn\\wpn_ak74_bump#");
    expect(gap).not.toContain("wpn\\wpn_ak74_bump ");
  });

  it("says a located pair the renderer refuses is never expanded to a png", () => {
    const gap: string = describeTextureChannelsGap(mockBumped(), EMPTY_TEXTURE_SURFACE, false) ?? "";

    expect(gap).toMatch(/cannot upload/);
    expect(gap).toMatch(/never expanded to a png/);
  });
});
