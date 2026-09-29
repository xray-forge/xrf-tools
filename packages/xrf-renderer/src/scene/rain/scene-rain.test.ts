import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { BufferGeometry, Mesh, Scene } from "three/webgpu";

import { IRendererRain } from "#/contract/weather/renderer-rain";
import { SceneRain } from "#/scene/rain/scene-rain";
import { ISceneRainStaging } from "#/scene/rain/scene-rain-staging";
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

function createRain(): { rain: SceneRain; textures: RendererTextures } {
  const textures: RendererTextures = new RendererTextures(
    () => {},
    () => {}
  );

  return { rain: new SceneRain(textures, new RainUniforms()), textures };
}

describe("SceneRain", () => {
  it("draws nothing until its build compiles, then every streak and a splash for each", () => {
    const { rain, textures } = createRain();
    const target = jest.spyOn(textures, "target");

    rain.take(RAIN);

    expect(rain.drawn).toBeNull();

    const staged: Nullable<ISceneRainStaging> = rain.takeStaged();

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

  it("drops a build taken again before it compiled", () => {
    const { rain } = createRain();

    rain.take(RAIN);

    const staged: Nullable<ISceneRainStaging> = rain.takeStaged();

    rain.take(null);
    staged?.commit();

    expect(rain.drawn).toBeNull();
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
