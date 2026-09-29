import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Mesh, Scene } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererThunder } from "#/contract/weather/renderer-thunder";
import { IRendererThunderboltStrike } from "#/contract/weather/renderer-thunderbolt-strike";
import { SceneThunder } from "#/scene/thunder/scene-thunder";
import { RendererTextures } from "#/texture/renderer-textures";
import { ThunderUniforms } from "#/uniforms/thunder-uniforms";

const GRADIENT = { draw: ERendererDraw.ADDED, opacity: 0.6, radius: [1, 1] as const, texture: "fx\\gradient" };

/** Two bolts of one model, the second glowing with a texture of its own at the top. */
const THUNDER: IRendererThunder = {
  animators: [],
  bolts: {
    one: { center: GRADIENT, color: null, model: 0, top: GRADIENT },
    two: { center: GRADIENT, color: null, model: 0, top: { ...GRADIENT, texture: "fx\\surge" } },
  },
  collections: { bolts: ["one", "two"] },
  models: [
    {
      draw: ERendererDraw.ADDED,
      indices: [0, 1, 2],
      positions: [0, 0, 0, 0, -1, 0, 0.1, -1, 0],
      texture: "fx\\fx_lightning",
      uvs: [0, 0, 0, 1, 1, 1],
    },
  ],
  settings: null,
};

const STRIKE: IRendererThunderboltStrike = {
  axes: [
    [300, 0, 0],
    [0, 300, 0],
    [0, 0, 300],
  ],
  bolt: "one",
  center: { extent: [300, 300], opacity: 0.5, position: [10, 50, 20] },
  position: [10, 200, 20],
  shift: 0,
  top: { extent: [150, 75], opacity: 0.5, position: [10, 200, 20] },
};

function createThunder(): SceneThunder {
  return new SceneThunder(
    new RendererTextures(
      () => {},
      () => {}
    ),
    new ThunderUniforms()
  );
}

/** Builds and compiles the thunder, returning what it draws while the strike shows. */
function strike(thunder: SceneThunder, shown: IRendererThunderboltStrike): Nullable<Scene> {
  thunder.take(THUNDER);
  thunder.takeStaged()?.commit();
  thunder.strike(shown);

  return thunder.drawn;
}

describe("SceneThunder", () => {
  it("draws nothing while no bolt strikes, and the striking bolt's model and glows alone while one does", () => {
    const thunder: SceneThunder = createThunder();

    thunder.take(THUNDER);
    thunder.takeStaged()?.commit();

    expect(thunder.drawn).toBeNull();

    const drawn: Nullable<Scene> = strike(thunder, STRIKE);
    const shown: Array<Mesh> = (drawn?.children ?? []).filter((it) => it.visible) as Array<Mesh>;

    // The model, and the first bolt's two glows; the second's top glow stays hidden.
    expect(shown.map((it: Mesh) => it.renderOrder)).toEqual([0, 1, 2]);
    expect(drawn?.children.length).toBe(4);

    thunder.strike(null);

    expect(thunder.drawn).toBeNull();
  });

  it("places the model by the strike's axes, from where it strikes", () => {
    const drawn: Nullable<Scene> = strike(createThunder(), STRIKE);
    const model: Mesh = drawn?.children[0] as Mesh;

    expect(model.matrixWorld.elements).toEqual([300, 0, 0, 0, 0, 300, 0, 0, 0, 0, 300, 0, 10, 200, 20, 1]);
  });
});
