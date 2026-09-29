import { describe, expect, it } from "@jest/globals";

import { ERendererEngine } from "#/contract/renderer-engine";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

describe("RendererUniforms", () => {
  // One engine names the shaders the scene is drawn by: its sky, its surfaces' reflections, its wet surfaces.
  it("draws by the lighting's engine", () => {
    const uniforms: RendererUniforms = new RendererUniforms();

    uniforms.light({ ...DEFAULT_RENDERER_LIGHTING, engine: ERendererEngine.EXTENDED });

    expect(uniforms.engine.extended.value).toBe(1);

    uniforms.light(DEFAULT_RENDERER_LIGHTING);

    expect(uniforms.engine.extended.value).toBe(0);
  });
});
