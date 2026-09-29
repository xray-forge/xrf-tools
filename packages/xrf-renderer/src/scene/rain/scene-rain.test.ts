import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { BufferGeometry, Mesh, Scene } from "three/webgpu";

import { IRendererRainfall } from "#/contract/renderer-rainfall";
import { IRendererRain } from "#/contract/weather/renderer-rain";
import { SceneRain } from "#/scene/rain/scene-rain";
import { ISceneBuildStaging } from "#/scene/staging/scene-build-staging";
import { RendererTextures } from "#/texture/renderer-textures";
import { RAIN_STREAKS, RainUniforms } from "#/uniforms/rain-uniforms";

/** A splash of two triangles, as small as `dm\rain.dm`. */
const RAIN: IRendererRain = {
  drop: {
    indices: [0, 1, 2, 2, 1, 3],
    positions: [-0.1, 0, -0.1, 0.1, 0, -0.1, -0.1, 0, 0.1, 0.1, 0, 0.1],
    texture: "fx\\fx_rainsplash1",
    uvs: [0, 0, 1, 0, 0, 1, 1, 1],
  },
  streak: "fx\\fx_rain",
};

/** Rain falling, which its draws are shown for. */
const RAINFALL: IRendererRainfall = { color: [0.5, 0.5, 0.5], density: 0.5, windDirection: 0, windVelocity: 0 };

function createRain(): { rain: SceneRain; textures: RendererTextures; uniforms: RainUniforms } {
  const textures: RendererTextures = new RendererTextures(
    () => {},
    () => {}
  );
  const uniforms: RainUniforms = new RainUniforms();

  uniforms.take(RAINFALL);

  return { rain: new SceneRain(textures, uniforms), textures, uniforms };
}

describe("SceneRain", () => {
  it("draws nothing until its build compiles, then every streak and a splash for each", () => {
    const { rain, textures } = createRain();
    const target = jest.spyOn(textures, "target");

    rain.take(RAIN);

    expect(rain.drawn).toBeNull();

    const staged: Nullable<ISceneBuildStaging> = rain.takeStaged();

    expect(rain.takeStaged()).toBeNull();

    staged?.commit();

    const drawn: Nullable<Scene> = rain.drawn;
    const [streaks, splashes] = (drawn?.children ?? []) as Array<Mesh>;

    expect(drawn).toBe(staged?.scene);
    expect(streaks.geometry.getAttribute("corner").count).toBe(RAIN_STREAKS * 4);
    expect((streaks.geometry.getIndex()?.count ?? 0) / 6).toBe(RAIN_STREAKS);
    expect(splashes.geometry.getAttribute("position").count).toBe(RAIN_STREAKS * 4);
    // Engine `z` negated into renderer space.
    expect(splashes.geometry.getAttribute("position").getZ(0)).toBeCloseTo(0.1, 6);
    expect(target.mock.calls.map((call) => call[0])).toEqual(["@weather/fx\\fx_rain", "@weather/fx\\fx_rainsplash1"]);
  });

  it("keeps drawing the last build while the next compiles, and lets it go for a weather without rain", () => {
    const { rain, textures } = createRain();
    const unbind = jest.spyOn(textures, "unbind");

    rain.take(RAIN);
    rain.takeStaged()?.commit();

    const first: Nullable<Scene> = rain.drawn;

    rain.take({ drop: null, streak: "fx\\fx_rain" });

    expect(rain.drawn).toBe(first);

    rain.takeStaged()?.commit();

    expect(rain.drawn).not.toBe(first);
    expect(rain.drawn?.children).toHaveLength(1);

    const disposed = jest.spyOn((rain.drawn?.children[0] as Mesh).geometry as BufferGeometry, "dispose");

    rain.take(null);

    expect(rain.drawn).toBeNull();
    expect(disposed).toHaveBeenCalled();
    expect(unbind).toHaveBeenCalled();
  });

  it("draws nothing while it does not rain", () => {
    const { rain, uniforms } = createRain();

    rain.take(RAIN);
    rain.takeStaged()?.commit();
    uniforms.take(null);

    expect(rain.drawn).toBeNull();

    uniforms.take(RAINFALL);

    expect(rain.drawn).not.toBeNull();
  });

  it("drops a build taken again before it compiled", () => {
    const { rain } = createRain();

    rain.take(RAIN);

    const staged: Nullable<ISceneBuildStaging> = rain.takeStaged();

    rain.take(null);
    staged?.commit();

    expect(rain.drawn).toBeNull();
  });

  // Three is still building its pipelines: taken down at once, the compile would bind what is gone.
  it("takes a build down once its compile ends, where another weather replaced it meanwhile", () => {
    const { rain } = createRain();

    rain.take(RAIN);

    const staged: Nullable<ISceneBuildStaging> = rain.takeStaged();
    const disposed = jest.spyOn((staged?.scene.children[0] as Mesh).geometry as BufferGeometry, "dispose");

    rain.take({ drop: null, streak: "fx\\fx_rain" });

    expect(disposed).not.toHaveBeenCalled();

    staged?.commit();

    expect(disposed).toHaveBeenCalled();
    expect(rain.drawn).toBeNull();
    expect(rain.takeStaged()?.scene.children).toHaveLength(1);
  });

  // A keyframe edited by hand sends the whole weather again, as often as a slider moves.
  it("builds nothing again for the same rain sent again", () => {
    const { rain } = createRain();

    rain.take(RAIN);
    rain.takeStaged()?.commit();

    const drawn: Nullable<Scene> = rain.drawn;

    rain.take(structuredClone(RAIN));

    expect(rain.takeStaged()).toBeNull();
    expect(rain.drawn).toBe(drawn);
  });
});
