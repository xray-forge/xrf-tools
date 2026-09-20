import { describe, expect, it } from "@jest/globals";
import { MeshStandardMaterial, WebGLProgramParametersWithUniforms } from "three";

import { createRenderMaterial } from "@/core/render/lib/surface/render-material";
import { IRenderSurface, OPAQUE_RENDER_SURFACE } from "@/core/render/lib/surface/render-surface";
import { applyXrayUnlitShading } from "@/core/render/lib/surface/render-unlit";

function shade(material: MeshStandardMaterial): string {
  const shader = {
    fragmentShader: "#include <opaque_fragment>\n#include <tonemapping_fragment>",
    uniforms: {},
    vertexShader: "",
  } as unknown as WebGLProgramParametersWithUniforms;

  material.onBeforeCompile(shader, null as never);

  return shader.fragmentShader;
}

describe("applyXrayUnlitShading", () => {
  it("writes the surface's own colour over whatever the lighting came to", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyXrayUnlitShading(material);

    expect(shade(material)).toContain("gl_FragColor = vec4( diffuseColor.rgb, diffuseColor.a );");
  });

  // Only the shading is replaced: the base texture, its detail modulation and its alpha are all settled before the
  // fragment is assigned, and tone mapping runs after it.
  it("keeps everything the fragment was built from and everything done to it after", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyXrayUnlitShading(material);

    const fragment: string = shade(material);

    expect(fragment).toContain("#include <opaque_fragment>");
    expect(fragment).toContain("#include <tonemapping_fragment>");
    expect(fragment.indexOf("gl_FragColor")).toBeLessThan(fragment.indexOf("#include <tonemapping_fragment>"));
  });
});

describe("createRenderMaterial", () => {
  const UNLIT: IRenderSurface = { ...OPAQUE_RENDER_SURFACE, isLit: false };

  // A composited pass is drawn after the light accumulation rather than into it, so lighting it composites the wrong
  // thing: a wall mark multiplied into a *lit* wall brightens it instead of being neutral.
  it("drops the lighting for a surface the engine draws unlit", () => {
    const material: MeshStandardMaterial = createRenderMaterial({ metalness: 0, roughness: 1 }, UNLIT);

    expect(material.customProgramCacheKey()).toContain("xray-unlit");
  });

  it("leaves a lit surface lit", () => {
    const material: MeshStandardMaterial = createRenderMaterial({ metalness: 0, roughness: 1 });

    expect(material.customProgramCacheKey()).not.toContain("xray-unlit");
  });
});
