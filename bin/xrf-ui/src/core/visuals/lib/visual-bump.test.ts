import { describe, expect, it } from "@jest/globals";
import { Matrix3, MeshStandardMaterial, RepeatWrapping, Texture, WebGLProgramParametersWithUniforms } from "three";

import {
  applyXrayBumpShading,
  decodeXrayBumpTexel,
  IVisualBumpShading,
  toLoadableBumps,
  XRAY_BINORMAL_ATTRIBUTE,
  XRAY_BUMP_GLOSS_GLSL,
  XRAY_BUMP_NORMAL_GLSL,
  XRAY_TANGENT_ATTRIBUTE,
} from "@/core/visuals/lib/visual-bump";
import { mockMaterialDescriptor, mockTextureDependency } from "@/fixtures/mocks/visual.mocks";

/**
 * The shader three.js would hand the patch, reduced to the includes the patch replaces.
 *
 * @returns Vertex and fragment sources carrying every marker, and an empty uniform set.
 */
function mockShader(): WebGLProgramParametersWithUniforms {
  return {
    uniforms: {},
    vertexShader: [
      "#include <common>",
      "void main() {",
      "#include <beginnormal_vertex>",
      "#include <skinnormal_vertex>",
      "#include <defaultnormal_vertex>",
      "}",
    ].join("\n"),
    fragmentShader: [
      "#include <common>",
      "void main() {",
      "#include <roughnessmap_fragment>",
      "#include <normal_fragment_begin>",
      "#include <normal_fragment_maps>",
      "}",
    ].join("\n"),
  } as unknown as WebGLProgramParametersWithUniforms;
}

describe("decodeXrayBumpTexel", () => {
  it("reconstructs the normal, gloss and height of a synthesized texel pair as sload.h does", () => {
    // Nu.wzy is (1.0, 0.5, 0.5) and the companion's error term is (-0.5, -0.5, 0.0), so the normal leans along x and z.
    // Height is the companion's fourth channel, not its third: the third is the error of the normal's z, which is
    // already spent correcting it above.
    const decoded = decodeXrayBumpTexel([0.6, 0.5, 0.5, 1.0], [0.5, 0.5, 1.0, 0.25]);

    expect(decoded.normal[0]).toBeCloseTo(0.5);
    expect(decoded.normal[1]).toBeCloseTo(0.0);
    expect(decoded.normal[2]).toBeCloseTo(0.5);
    expect(decoded.gloss).toBeCloseTo(0.36);
    expect(decoded.height).toBeCloseTo(0.25);
  });

  it("decodes the flat dummy pair to a normal pointing straight out", () => {
    // What the engine draws for a bump it cannot find: no tilt, no gloss to speak of.
    const decoded = decodeXrayBumpTexel([0.0, 0.5, 0.5, 0.5], [0.5, 0.5, 0.5, 0.5]);

    expect(decoded.normal).toEqual([0, 0, 0]);
    expect(decoded.gloss).toBe(0);
  });
});

describe("applyXrayBumpShading", () => {
  it("patches the standard shaders with the engine's decode and the authored basis", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();
    const shader: WebGLProgramParametersWithUniforms = mockShader();

    applyXrayBumpShading(material, { bump: new Texture(), companion: new Texture() });
    material.onBeforeCompile(shader, null as never);

    expect(shader.fragmentShader).toContain(XRAY_BUMP_NORMAL_GLSL);
    expect(shader.fragmentShader).toContain(XRAY_BUMP_GLOSS_GLSL);
    expect(shader.fragmentShader).toContain("#include <roughnessmap_fragment>");
    expect(shader.fragmentShader).toContain("#include <normal_fragment_maps>");
    expect(shader.vertexShader).toContain(`attribute vec3 ${XRAY_TANGENT_ATTRIBUTE};`);
    expect(shader.vertexShader).toContain(`attribute vec3 ${XRAY_BINORMAL_ATTRIBUTE};`);
    expect(shader.vertexShader).toContain("skinMatrix * vec4( xrayObjectTangent, 0.0 )");
    expect(Object.keys(shader.uniforms).sort()).toEqual([
      "xrayBump",
      "xrayBumpEnabled",
      "xrayBumpUvTransform",
      "xrayBumpX",
    ]);
    expect(material.customProgramCacheKey()).toBe("xray-bump");
  });

  it("switches the surface through a uniform rather than a recompile", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();
    const shader: WebGLProgramParametersWithUniforms = mockShader();
    const shading: IVisualBumpShading = applyXrayBumpShading(material, {
      bump: new Texture(),
      companion: new Texture(),
    });

    material.onBeforeCompile(shader, null as never);

    expect(shader.uniforms.xrayBumpEnabled.value).toBe(1);

    shading.setEnabled(false);

    expect(shader.uniforms.xrayBumpEnabled.value).toBe(0);
  });

  it("samples the pair through the same uv transform as the texture it shades", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();
    const shader: WebGLProgramParametersWithUniforms = mockShader();
    const shading: IVisualBumpShading = applyXrayBumpShading(material, {
      bump: new Texture(),
      companion: new Texture(),
    });

    material.onBeforeCompile(shader, null as never);

    // Identity until told otherwise, so an untiled surface samples exactly the uvs its geometry carries.
    expect(shader.vertexShader).toContain("vXrayUv = ( xrayBumpUvTransform * vec3( uv, 1.0 ) ).xy;");
    expect((shader.uniforms.xrayBumpUvTransform.value as Matrix3).elements).toEqual(new Matrix3().elements);

    const tiled: Texture = new Texture();

    tiled.wrapS = RepeatWrapping;
    tiled.wrapT = RepeatWrapping;
    tiled.repeat.set(4, 4);
    tiled.updateMatrix();

    shading.setUvTransform(tiled.matrix);

    expect((shader.uniforms.xrayBumpUvTransform.value as Matrix3).elements).toEqual(tiled.matrix.elements);
  });
});

describe("toLoadableBumps", () => {
  it("keeps the submeshes whose material located both halves of the pair, addressed by those files", () => {
    const declared = mockMaterialDescriptor();
    const halfMissing = mockMaterialDescriptor({ outcome: "missing" });

    halfMissing.bump!.companion.resolution = { kind: "missing", roots: ["C:\\gamedata"] };

    expect(
      toLoadableBumps(
        [
          mockTextureDependency({ submeshIndex: 0, reference: "wpn\\wpn_ak74" }),
          mockTextureDependency({ submeshIndex: 1, reference: "wpn\\wpn_half" }),
          mockTextureDependency({ submeshIndex: 2, reference: "wpn\\wpn_flat" }),
        ],
        {
          ["wpn\\wpn_ak74"]: declared,
          ["wpn\\wpn_half"]: halfMissing,
          ["wpn\\wpn_flat"]: { ...declared, declaration: { kind: "noDescriptor" }, bump: null, outcome: "flat" },
        }
      )
    ).toEqual([
      {
        submeshIndex: 0,
        bump: "textures\\wpn\\wpn_ak74_bump.dds",
        companion: "textures\\wpn\\wpn_ak74_bump#.dds",
      },
    ]);
  });
});
