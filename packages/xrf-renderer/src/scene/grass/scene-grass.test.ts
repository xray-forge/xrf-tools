import { describe, expect, it, jest } from "@jest/globals";
import { BufferAttribute, Mesh, PerspectiveCamera, WebGPURenderer } from "three/webgpu";

import { DEFAULT_RENDERER_GRASS_SETTINGS, IRendererGrassSettings } from "#/contract/renderer-grass-settings";
import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererGrass, RENDERER_GRASS_SLOT_WORDS } from "#/contract/scene/renderer-grass";
import { adoptRendererConventions } from "#/internals/camera-conventions";
import { SceneGrass } from "#/scene/grass/scene-grass";
import { ISceneGrassStaging } from "#/scene/grass/scene-grass-staging";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** The engine's grass, reaching twice as far: a build of it outgrows the engine's. */
const FAR: IRendererGrassSettings = { ...DEFAULT_RENDERER_GRASS_SETTINGS, radius: 98 };

interface IGrassFixture {
  grass: SceneGrass;
  renderer: WebGPURenderer;
  view: PerspectiveCamera;
  /** Every buffer retired so far. */
  retired: Set<BufferAttribute>;
}

function createGrass(): IRendererGrass {
  return {
    bins: new Uint32Array(1),
    grid: new Uint32Array(16),
    models: [
      {
        height: 1,
        indices: new Uint16Array([0, 1, 2]),
        isWaving: false,
        maxScale: 1,
        minScale: 1,
        positions: new Float32Array(9),
        radius: 1,
        surface: { draw: ERendererDraw.CUT_OUT, textures: {} },
        uvs: new Float32Array(6),
      },
    ],
    offsetX: 2,
    offsetZ: 2,
    sizeX: 4,
    sizeZ: 4,
    slots: new Uint32Array(RENDERER_GRASS_SLOT_WORDS),
    triangles: new Float32Array(9),
  };
}

function createFixture(): IGrassFixture {
  const uniforms: RendererUniforms = new RendererUniforms();
  const retired: Set<BufferAttribute> = new Set();
  const grass: SceneGrass = new SceneGrass(
    new RendererTextures(
      () => {},
      () => {}
    ),
    uniforms
  );
  const view: PerspectiveCamera = new PerspectiveCamera(90, 1, 0.1, 1000);

  adoptRendererConventions(view);
  view.updateMatrixWorld();
  jest
    .spyOn(uniforms.retirement, "retire")
    .mockImplementation((attributes: Iterable<BufferAttribute>) =>
      Array.from(attributes).forEach((attribute: BufferAttribute) => retired.add(attribute))
    );
  grass.put(createGrass());

  return { grass, renderer: { compute: jest.fn() } as unknown as WebGPURenderer, retired, view };
}

/** The draw arguments a staged build's draws read, which are the level's. */
function toLevelArguments(staging: ISceneGrassStaging): BufferAttribute {
  return ((staging.scene.children[0] as Mesh).geometry as unknown as { indirect: BufferAttribute }).indirect;
}

describe("SceneGrass", () => {
  it("draws nothing until its first build has compiled, handing that build over once", () => {
    const { grass, renderer, view }: IGrassFixture = createFixture();

    expect(grass.plant(renderer, view, DEFAULT_RENDERER_GRASS_SETTINGS)).toBeNull();

    const staging: ISceneGrassStaging = grass.takeStaged() as ISceneGrassStaging;

    expect(grass.takeStaged()).toBeNull();
    expect(grass.plant(renderer, view, DEFAULT_RENDERER_GRASS_SETTINGS)).toBeNull();
    expect(renderer.compute).not.toHaveBeenCalled();

    staging.commit();

    expect(grass.plant(renderer, view, DEFAULT_RENDERER_GRASS_SETTINGS)).toBe(staging.scene);
    expect(renderer.compute).toHaveBeenCalledTimes(1);
    expect(grass.takeStaged()).toBeNull();
  });

  it("hands an abandoned build over again", () => {
    const { grass, renderer, view }: IGrassFixture = createFixture();

    grass.plant(renderer, view, DEFAULT_RENDERER_GRASS_SETTINGS);

    const staging: ISceneGrassStaging = grass.takeStaged() as ISceneGrassStaging;

    staging.abandon();

    expect(grass.takeStaged()?.scene).toBe(staging.scene);
  });

  it("keeps planting within its own ring while a larger build compiles, and lets it go once that is in", () => {
    const { grass, renderer, retired, view }: IGrassFixture = createFixture();

    grass.plant(renderer, view, DEFAULT_RENDERER_GRASS_SETTINGS);

    const first: ISceneGrassStaging = grass.takeStaged() as ISceneGrassStaging;
    const level: BufferAttribute = toLevelArguments(first);

    first.commit();

    expect(grass.plant(renderer, view, FAR)).toBe(first.scene);

    const larger: ISceneGrassStaging = grass.takeStaged() as ISceneGrassStaging;

    expect(larger.scene).not.toBe(first.scene);
    // The engine's radius is 49 metres, 25 slots of two each way: its build holds no more.
    expect(grass.plant(renderer, view, FAR)).toBe(first.scene);
    expect(grass.uniforms.reach.value).toBeGreaterThan(25);

    const shrunk: number = retired.size;

    larger.commit();

    expect(grass.plant(renderer, view, FAR)).toBe(larger.scene);
    expect(retired.size).toBeGreaterThan(shrunk);
    expect(retired.has(level)).toBe(false);
  });

  it("builds again a build not yet taken to compile once the settings outgrow it, letting it go", () => {
    const { grass, renderer, retired, view }: IGrassFixture = createFixture();

    grass.plant(renderer, view, DEFAULT_RENDERER_GRASS_SETTINGS);

    const kept: number = retired.size;

    grass.plant(renderer, view, FAR);

    const staging: ISceneGrassStaging = grass.takeStaged() as ISceneGrassStaging;

    expect(retired.size).toBeGreaterThan(kept);

    staging.commit();
    grass.plant(renderer, view, FAR);

    expect(grass.takeStaged()).toBeNull();
  });

  it("builds nothing again for settings its build still holds", () => {
    const { grass, renderer, view }: IGrassFixture = createFixture();

    grass.plant(renderer, view, FAR);
    (grass.takeStaged() as ISceneGrassStaging).commit();
    grass.plant(renderer, view, DEFAULT_RENDERER_GRASS_SETTINGS);

    expect(grass.takeStaged()).toBeNull();
  });

  it("holds the level's buffers back while a build binding them compiles, and lets them go once it ends", () => {
    const { grass, renderer, retired, view }: IGrassFixture = createFixture();

    grass.plant(renderer, view, DEFAULT_RENDERER_GRASS_SETTINGS);

    const staging: ISceneGrassStaging = grass.takeStaged() as ISceneGrassStaging;
    const level: BufferAttribute = toLevelArguments(staging);

    grass.release();

    expect(retired.has(level)).toBe(false);

    staging.commit();

    expect(retired.has(level)).toBe(true);
    expect(grass.plant(renderer, view, DEFAULT_RENDERER_GRASS_SETTINGS)).toBeNull();
  });

  it("lets the level's buffers go at once where nothing compiling binds them", () => {
    const { grass, renderer, retired, view }: IGrassFixture = createFixture();

    grass.plant(renderer, view, DEFAULT_RENDERER_GRASS_SETTINGS);

    const staging: ISceneGrassStaging = grass.takeStaged() as ISceneGrassStaging;
    const level: BufferAttribute = toLevelArguments(staging);

    staging.commit();
    grass.release();

    expect(retired.has(level)).toBe(true);
  });
});
