import { describe, expect, it, jest } from "@jest/globals";

import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { SceneSky } from "#/scene/sky/scene-sky";
import { RendererTextures } from "#/texture/renderer-textures";
import { CloudUniforms } from "#/uniforms/cloud-uniforms";
import { SkyUniforms } from "#/uniforms/sky-uniforms";

function createSky(uploaded: Set<string>): { scene: SceneSky; sky: SkyUniforms } {
  const textures: RendererTextures = new RendererTextures(
    () => {},
    () => {}
  );
  const sky: SkyUniforms = new SkyUniforms();

  jest.spyOn(textures, "target").mockImplementation(() => {});
  jest.spyOn(textures, "isUploaded").mockImplementation((key: string) => uploaded.has(key));

  return { scene: new SceneSky(textures, { clouds: new CloudUniforms(), sky }), sky };
}

describe("SceneSky", () => {
  it("blends out a keyframe whose sky is still going up, and blends as asked once both are up", () => {
    const uploaded: Set<string> = new Set(["a", "a#small"]);
    const { scene, sky } = createSky(uploaded);

    scene.take({
      ...DEFAULT_RENDERER_LIGHTING.sky,
      blend: 0.4,
      environments: ["a#small", "b#small"],
      textures: ["a", "b"],
    });
    scene.update();

    expect(sky.blend.value).toBe(0);

    uploaded.add("b");
    scene.update();

    // Its irradiance cube is not up yet either.
    expect(sky.blend.value).toBe(0);

    uploaded.add("b#small");
    scene.update();

    expect(sky.blend.value).toBe(0.4);

    uploaded.delete("a");
    scene.update();

    expect(sky.blend.value).toBe(1);
  });

  it("blends as asked while neither is up, both drawing their placeholders", () => {
    const { scene, sky } = createSky(new Set());

    scene.take({
      ...DEFAULT_RENDERER_LIGHTING.sky,
      blend: 0.4,
      environments: ["a#small", "b#small"],
      textures: ["a", "b"],
    });
    scene.update();

    expect(sky.blend.value).toBe(0.4);
  });
});
