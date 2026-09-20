import { describe, expect, it } from "@jest/globals";
import { MeshStandardMaterial, WebGLProgramParametersWithUniforms } from "three";

import { applyXrayHemiShading, XRAY_HEMI_CHANNEL } from "@/core/render/lib/surface/render-baked";
import { applyXrayDetailShading } from "@/core/render/lib/surface/render-detail";

function mockShader(): WebGLProgramParametersWithUniforms {
  return {
    uniforms: {},
    vertexShader: ["#include <common>", "void main() {", "}"].join("\n"),
    fragmentShader: [
      "#include <common>",
      "void main() {",
      "#include <map_fragment>",
      "#include <aomap_fragment>",
      "}",
    ].join("\n"),
  } as WebGLProgramParametersWithUniforms;
}

function patched(material: MeshStandardMaterial): WebGLProgramParametersWithUniforms {
  const shader: WebGLProgramParametersWithUniforms = mockShader();

  applyXrayHemiShading(material);
  material.onBeforeCompile(shader, null as never);

  return shader;
}

describe("applyXrayHemiShading", () => {
  // `get_hemi(lm) = lm.a` (`shaders/r2/common.h`), where three.js reads `.r`. The channel is the whole difference,
  // and reading the wrong one takes the occlusion off a level entirely.
  it("reads the channel the engine reads out of the occlusion map", () => {
    const shader: WebGLProgramParametersWithUniforms = patched(new MeshStandardMaterial());

    expect(shader.fragmentShader).toContain(`texture2D( aoMap, vAoMapUv ).${XRAY_HEMI_CHANNEL}`);
    expect(shader.fragmentShader).not.toContain("#include <aomap_fragment>");
    expect(shader.fragmentShader).not.toContain("texture2D( aoMap, vAoMapUv ).r");
  });

  // Three.js caches a compiled program by its parameters and this key. Two materials agreeing on every parameter but
  // patched differently would be handed each other's shader, which is a detailed surface drawn undetailed or worse.
  it("names itself in the program key, beside whatever was patched in before it", () => {
    const plain: MeshStandardMaterial = new MeshStandardMaterial();
    const detailed: MeshStandardMaterial = new MeshStandardMaterial();

    applyXrayDetailShading(detailed);
    applyXrayHemiShading(plain);
    applyXrayHemiShading(detailed);

    expect(plain.customProgramCacheKey()).toContain("xray-hemi");
    expect(detailed.customProgramCacheKey()).toContain("xray-hemi");
    expect(detailed.customProgramCacheKey()).not.toBe(plain.customProgramCacheKey());
  });

  it("leaves a patch applied before it in place", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyXrayDetailShading(material);

    const shader: WebGLProgramParametersWithUniforms = patched(material);

    expect(shader.uniforms.xrayDetail).toBeDefined();
    expect(shader.fragmentShader).toContain("xrayDetailFactor");
    expect(shader.fragmentShader).toContain(`texture2D( aoMap, vAoMapUv ).${XRAY_HEMI_CHANNEL}`);
  });
});
