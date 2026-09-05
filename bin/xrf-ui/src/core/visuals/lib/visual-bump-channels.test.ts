import { describe, expect, it } from "@jest/globals";
import { Texture } from "three";

import {
  XRAY_BUMP_GLOSS_GLSL,
  XRAY_BUMP_HEIGHT_GLSL,
  XRAY_BUMP_NORMAL_GLSL,
} from "@/core/visuals/lib/visual-bump";
import { createXrayBumpChannels, EVisualBumpView, IVisualBumpChannels } from "@/core/visuals/lib/visual-bump-channels";

function mockChannels(): IVisualBumpChannels {
  return createXrayBumpChannels({ bump: new Texture(), companion: new Texture() });
}

describe("createXrayBumpChannels", () => {
  it("reconstructs through the same expressions the lit surface is shaded by", () => {
    // The whole point of these views is to check the decode. A second spelling of it here would check itself.
    const { material } = mockChannels();

    expect(material.fragmentShader).toContain(XRAY_BUMP_NORMAL_GLSL);
    expect(material.fragmentShader).toContain(XRAY_BUMP_GLOSS_GLSL);
    expect(material.fragmentShader).toContain(XRAY_BUMP_HEIGHT_GLSL);
  });

  it("samples both halves of the pair it was given", () => {
    const bump: Texture = new Texture();
    const companion: Texture = new Texture();
    const { material } = createXrayBumpChannels({ bump, companion });

    expect(material.uniforms.xrayBump.value).toBe(bump);
    expect(material.uniforms.xrayBumpX.value).toBe(companion);
  });

  it("switches view through a uniform, so a tile costs one draw and no recompile", () => {
    const channels: IVisualBumpChannels = mockChannels();
    const seen: Array<number> = [];

    for (const view of Object.values(EVisualBumpView)) {
      channels.setView(view);
      seen.push(channels.material.uniforms.xrayBumpView.value as number);
    }

    // One index per view, and every view reachable: a duplicate would draw one plane twice and hide another.
    expect(new Set(seen).size).toBe(Object.values(EVisualBumpView).length);
  });

  it("reads the row a file stores first as the row at the top", () => {
    // X-Ray stores rows top first and neither loader flips one, so a tile drawn straight from three.js' generated uvs
    // shows every plane upside down against the picture of the same file.
    const { material } = mockChannels();

    expect(material.fragmentShader).toContain("vec2( vXrayUv.x, 1.0 - vXrayUv.y )");
  });

  it("writes what it read, with no colour transform in the way", () => {
    // A plane of packed numbers shown through an output encoding is no longer the plane.
    const { material } = mockChannels();

    expect(material.fragmentShader).not.toContain("colorspace_fragment");
    expect(material.fragmentShader).not.toContain("linearToOutputTexel");
  });
});
