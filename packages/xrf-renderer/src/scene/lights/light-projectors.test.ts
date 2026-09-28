import { describe, expect, it } from "@jest/globals";
import { TextureNode } from "three/webgpu";

import { ERendererLightKind } from "#/contract/scene/renderer-light";
import { IRendererPointLight } from "#/contract/scene/renderer-point-light";
import { IRendererSpotLight } from "#/contract/scene/renderer-spot-light";
import { LightProjectors, MAX_PROJECTORS } from "#/scene/lights/light-projectors";
import { RendererTextures } from "#/texture/renderer-textures";

const POINT: IRendererPointLight = {
  animatorScale: 0,
  color: [1, 1, 1],
  isLevel: false,
  isShadowed: false,
  kind: ERendererLightKind.POINT,
  near: 0,
  position: [0, 0, 0],
  range: 4,
};

function createSpot(projector?: string): IRendererSpotLight {
  return {
    ...POINT,
    cone: Math.PI / 2,
    direction: [0, -1, 0],
    kind: ERendererLightKind.SPOT,
    projector,
    right: [1, 0, 0],
  };
}

function createProjectors(): LightProjectors {
  return new LightProjectors(
    new RendererTextures(
      () => {},
      () => {}
    )
  );
}

describe("LightProjectors", () => {
  it("gives each projector a slot in the order the spots first name it, and none to a point or a spot without", () => {
    const projectors: LightProjectors = createProjectors();
    const [first, second, again] = [createSpot("a"), createSpot("b"), createSpot("a")];

    projectors.put([POINT, first, createSpot(), second, again]);

    expect([first, second, again].map((light: IRendererSpotLight) => projectors.getSlot(light))).toEqual([0, 1, 0]);
    expect(projectors.getSlot(POINT)).toBe(-1);
    expect(projectors.getSlot(createSpot())).toBe(-1);
    expect(projectors.samplers).toHaveLength(MAX_PROJECTORS);
  });

  it("lights a spot white past the slots", () => {
    const projectors: LightProjectors = createProjectors();
    const spots: Array<IRendererSpotLight> = Array.from({ length: MAX_PROJECTORS + 1 }, (_: unknown, index: number) =>
      createSpot(`${index}`)
    );

    projectors.put(spots);

    expect(projectors.getSlot(spots[MAX_PROJECTORS - 1])).toBe(MAX_PROJECTORS - 1);
    expect(projectors.getSlot(spots[MAX_PROJECTORS])).toBe(-1);
  });

  it("binds its slots again only when the projectors named change, and to white once let go", () => {
    const projectors: LightProjectors = createProjectors();

    projectors.put([createSpot("a")]);

    const { version } = projectors;
    const samplers: ReadonlyArray<TextureNode> = projectors.samplers;

    projectors.put([createSpot("a"), POINT]);

    expect(projectors.version).toBe(version);
    expect(projectors.samplers).toBe(samplers);

    projectors.put([createSpot("b")]);

    expect(projectors.version).toBe(version + 1);
    expect(projectors.samplers).not.toBe(samplers);

    projectors.release();

    expect(projectors.version).toBe(version + 2);
    expect(projectors.getSlot(createSpot("b"))).toBe(-1);
  });
});
