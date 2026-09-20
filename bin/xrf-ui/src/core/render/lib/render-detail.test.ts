import { describe, expect, it } from "@jest/globals";
import { MeshStandardMaterial, Texture, WebGLProgramParametersWithUniforms } from "three";

import {
  applyXrayDetailShading,
  IXrayDetailShading,
  toXrayDetailFactor,
  XRAY_DETAIL_FACTOR_GLSL,
} from "@/core/render/lib/render-detail";

function mockShader(): WebGLProgramParametersWithUniforms {
  return {
    uniforms: {},
    vertexShader: ["#include <common>", "void main() {", "}"].join("\n"),
    fragmentShader: ["#include <common>", "void main() {", "#include <map_fragment>", "}"].join("\n"),
  } as WebGLProgramParametersWithUniforms;
}

function patched(): {
  material: MeshStandardMaterial;
  shader: WebGLProgramParametersWithUniforms;
  shading: IXrayDetailShading;
} {
  const material: MeshStandardMaterial = new MeshStandardMaterial();
  const shader: WebGLProgramParametersWithUniforms = mockShader();
  const shading: IXrayDetailShading = applyXrayDetailShading(material);

  material.onBeforeCompile(shader, null as never);

  return { material, shader, shading };
}

describe("toXrayDetailFactor", () => {
  // Mid grey is the neutral element of a doubled modulate, which is what makes an average detail texture leave the
  // surface it lies over at the brightness the base texture authored.
  it("leaves mid grey neutral and doubles what is brighter", () => {
    expect(toXrayDetailFactor(0.5)).toBeCloseTo(1);
    expect(toXrayDetailFactor(1)).toBeCloseTo(2);
    expect(toXrayDetailFactor(0)).toBeCloseTo(0);
  });

  // The deferred renderer applies the modulation whatever the distance. R1 fades it out by `r__dtex_range`, and
  // reproducing that fade here drew a level detailed for fifty metres and flat past it - which is what the report
  // "details show only at the closest distance" was.
  it("does not fade with distance, because the renderer this reproduces does not", () => {
    expect(toXrayDetailFactor(1)).toBeCloseTo(2);
  });
});

describe("applyXrayDetailShading", () => {
  it("modulates the diffuse where the base texture just produced it", () => {
    const { material, shader } = patched();

    expect(shader.fragmentShader).toContain(XRAY_DETAIL_FACTOR_GLSL);
    // After the base texture rather than before it: there is nothing to modulate until `map_fragment` has run.
    expect(shader.fragmentShader.indexOf("#include <map_fragment>")).toBeLessThan(
      shader.fragmentShader.indexOf("diffuseColor.rgb *= xrayDetailFactor;")
    );
    // Only where three.js carries the base coordinate, which is the coordinate the detail is laid out in.
    expect(shader.fragmentShader).toContain("#ifdef USE_MAP");
    expect(Object.keys(shader.uniforms).sort()).toEqual(["xrayDetail", "xrayDetailEnabled", "xrayDetailScale"]);
    expect(material.customProgramCacheKey()).toBe("xray-detail");
  });

  it("draws undetailed until a texture is applied", () => {
    // A material is built with the patch as soon as its blender says the surface is detailed, which is before its
    // texture has been read. Until it arrives the surface is drawn exactly as an undetailed one.
    const { shader } = patched();

    expect(shader.uniforms.xrayDetailEnabled.value).toBe(0);
  });

  it("applies a texture and its tiling through uniforms rather than a recompile", () => {
    const { material, shader, shading } = patched();
    const texture: Texture = new Texture();
    // `needsUpdate` is write-only and bumps this, which is what three.js rebuilds a program on.
    const version: number = material.version;

    shading.setDetail({ scale: 150, texture });

    expect(shader.uniforms.xrayDetail.value).toBe(texture);
    expect(shader.uniforms.xrayDetailEnabled.value).toBe(1);
    expect(shader.uniforms.xrayDetailScale.value).toBe(150);
    expect(material.version).toBe(version);
  });

  // `tcdbump` is `tcdh * dt_params` and nothing else: no view position reaches the detail coordinate.
  it("lays the detail out on the base coordinate alone", () => {
    const { shader } = patched();

    expect(shader.fragmentShader).toContain("vMapUv * xrayDetailScale");
    expect(shader.fragmentShader).not.toContain("vViewPosition");
  });

  it("takes the modulation off without unbinding what it was modulating with", () => {
    const { shader, shading } = patched();

    shading.setDetail({ scale: 8, texture: new Texture() });
    shading.setDetail(null);

    expect(shader.uniforms.xrayDetailEnabled.value).toBe(0);
    expect(shader.uniforms.xrayDetail.value).toBeNull();
  });
});
