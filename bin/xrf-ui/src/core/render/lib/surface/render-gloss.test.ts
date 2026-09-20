import { describe, expect, it } from "@jest/globals";
import { MeshStandardMaterial, WebGLProgramParametersWithUniforms } from "three";

import { applyXrayGlossShading, XRAY_DEFAULT_GLOSS, XRAY_GLOSS_VARIABLE } from "@/core/render/lib/surface/render-gloss";

/** The two chunks the patch works over, in the order the physical fragment shader includes them. */
const FRAGMENT: string = ["#include <clipping_planes_fragment>", "#include <lights_physical_fragment>"].join("\n");

function shade(gloss?: number): string {
  const material: MeshStandardMaterial = new MeshStandardMaterial();
  const shader = {
    fragmentShader: FRAGMENT,
    uniforms: {},
    vertexShader: "",
  } as unknown as WebGLProgramParametersWithUniforms;

  applyXrayGlossShading(material, gloss);
  material.onBeforeCompile(shader, null as never);

  return shader.fragmentShader;
}

describe("applyXrayGlossShading", () => {
  it("is the engine's own default, two of two hundred and fifty five", () => {
    expect(XRAY_DEFAULT_GLOSS).toBeCloseTo(0.00784, 5);
  });

  it("compiles the gloss in as a constant before anything reads it", () => {
    expect(shade()).toContain(`float ${XRAY_GLOSS_VARIABLE} = 0.007843;`);
  });

  it("takes the gloss a surface declares over the default", () => {
    expect(shade(0.5)).toContain(`float ${XRAY_GLOSS_VARIABLE} = 0.500000;`);
  });

  it("declares the gloss before the shading that reads it", () => {
    const fragment: string = shade();

    expect(fragment.indexOf(`float ${XRAY_GLOSS_VARIABLE}`)).toBeLessThan(fragment.indexOf("material.specularColor"));
  });

  /**
   * All three, because the first two are the reflectance at normal incidence and the third is what the Fresnel term
   * climbs towards: scaling anything less leaves a surface at full strength exactly where the sun grazes it, which is
   * the sheen this exists to take off.
   */
  it("scales the whole specular lobe by it", () => {
    const fragment: string = shade();

    expect(fragment).toContain(`material.specularColor *= ${XRAY_GLOSS_VARIABLE};`);
    expect(fragment).toContain(`material.specularColorBlended *= ${XRAY_GLOSS_VARIABLE};`);
    expect(fragment).toContain(`material.specularF90 *= ${XRAY_GLOSS_VARIABLE};`);
  });

  it("keeps the chunk it shades after", () => {
    expect(shade()).toContain("#include <lights_physical_fragment>");
  });

  // Two glosses are two programs, and a key that named neither would hand one surface the other's shader.
  it("keys a program by the gloss compiled into it", () => {
    const one: MeshStandardMaterial = new MeshStandardMaterial();
    const other: MeshStandardMaterial = new MeshStandardMaterial();

    applyXrayGlossShading(one);
    applyXrayGlossShading(other, 0.5);

    expect(one.customProgramCacheKey()).not.toBe(other.customProgramCacheKey());
  });
});
