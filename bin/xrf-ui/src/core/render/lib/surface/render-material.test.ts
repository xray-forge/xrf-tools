import { describe, expect, it } from "@jest/globals";
import { MeshStandardMaterial } from "three";

import { applyRenderSurface, createRenderMaterial } from "@/core/render/lib/surface/render-material";
import { IRenderSurface, OPAQUE_RENDER_SURFACE } from "@/core/render/lib/surface/render-surface";

const CUT_OUT: IRenderSurface = { ...OPAQUE_RENDER_SURFACE, alphaTest: 200 / 255 };
const BLENDED: IRenderSurface = {
  ...OPAQUE_RENDER_SURFACE,
  alphaTest: 32 / 255,
  isDepthWritten: false,
  isTransparent: true,
};

describe("applyRenderSurface", () => {
  // The one place render state is set, for a model submesh and a level surface alike. Two places for it is how the
  // level viewer came to draw every cut-out surface solid while the mesh viewer drew the same shader correctly.
  it("writes the three states a shader's answer decides, and nothing else", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial({ color: 0x336699 });

    applyRenderSurface(material, CUT_OUT);

    expect(material).toMatchObject({ alphaTest: CUT_OUT.alphaTest, depthWrite: true, transparent: false });
    expect(material.color.getHex()).toBe(0x336699);
  });

  it("stops a blended surface writing depth, so two of them show through one another", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderSurface(material, BLENDED);

    expect(material).toMatchObject({ depthWrite: false, transparent: true });
  });

  it("returns a material to opaque, which is what the comparison toggle asks for", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderSurface(material, BLENDED);
    applyRenderSurface(material, OPAQUE_RENDER_SURFACE);

    expect(material).toMatchObject({ alphaTest: 0, depthWrite: true, transparent: false });
  });
});

describe("createRenderMaterial", () => {
  it("dresses a material before it has ever drawn", () => {
    // The shader's answer arrives with the description rather than with the texture, so a cut-out surface is never
    // shown solid for a frame and then corrected.
    const material: MeshStandardMaterial = createRenderMaterial({ metalness: 0, roughness: 0.9 }, CUT_OUT);

    expect(material.alphaTest).toBeCloseTo(200 / 255);
    expect(material).toMatchObject({ metalness: 0, roughness: 0.9 });
  });

  it("draws a surface with nothing said about it the way the engine draws an unresolved shader", () => {
    const material: MeshStandardMaterial = createRenderMaterial({ metalness: 0.05, roughness: 0.75 });

    expect(material).toMatchObject({ alphaTest: 0, depthWrite: true, transparent: false });
    expect(material.color.getHex()).toBe(0xffffff);
  });
});
