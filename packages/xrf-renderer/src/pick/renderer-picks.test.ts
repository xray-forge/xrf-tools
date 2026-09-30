import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";
import {
  BufferGeometry,
  Camera,
  Color,
  Material,
  Mesh,
  MeshBasicNodeMaterial,
  PerspectiveCamera,
  Scene,
  WebGPURenderer,
} from "three/webgpu";

import { IRendererHit } from "#/contract/scene/renderer-hit";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { RendererPicks } from "#/pick/renderer-picks";
import { IPickTexel } from "#/scene/object/pick-texel";
import { RendererScene } from "#/scene/renderer-scene";
import { EPickKind } from "#/shader/pick-kind";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** What each render of the fake drew: every mesh's material and whether it was shown, and the camera. */
interface IDrawn {
  camera: Camera;
  meshes: Array<{ material: Material | Array<Material>; isVisible: boolean }>;
}

/** Just enough of a renderer for a pick: it records what it drew and reads back the texel given. */
function createRenderer(texel: ArrayLike<number>): { renderer: WebGPURenderer; drawn: Array<IDrawn> } {
  const drawn: Array<IDrawn> = [];
  const renderer = {
    clear: jest.fn(),
    getClearAlpha: () => 1,
    getClearColor: (color: Color) => color.set(0x123456),
    getRenderTarget: () => null,
    readRenderTargetPixelsAsync: jest.fn(async () => Float32Array.from(texel)),
    render: (scene: Scene, camera: Camera) =>
      drawn.push({
        camera,
        meshes: (scene.children as Array<Mesh>).map((mesh: Mesh) => ({
          isVisible: mesh.visible,
          material: mesh.material,
        })),
      }),
    setClearColor: jest.fn(),
    setRenderTarget: jest.fn(),
    sortObjects: true,
  };

  return { drawn, renderer: renderer as unknown as WebGPURenderer };
}

/** A camera at the origin looking down -z, over a view twice as wide as tall. */
function createCamera(): PerspectiveCamera {
  const camera: PerspectiveCamera = new PerspectiveCamera(60, 2, 0.1, 100);

  camera.updateProjectionMatrix();
  camera.updateMatrixWorld();

  return camera;
}

/** Lets the read back's promise land. */
async function landRead(): Promise<void> {
  for (let turn: number = 0; turn < 3; turn += 1) {
    await Promise.resolve();
  }
}

describe("RendererPicks", () => {
  const uniforms: RendererUniforms = new RendererUniforms();

  function createSurface(): SurfaceNodeMaterial {
    const material: SurfaceNodeMaterial = new SurfaceNodeMaterial(uniforms.staticDraws, uniforms.treeWind);

    material.pick = new SurfaceNodeMaterial(uniforms.staticDraws, uniforms.treeWind);

    return material;
  }

  it("draws every surface by its twin and hides what has none, then puts back both and the renderer", async () => {
    const surface: SurfaceNodeMaterial = createSurface();
    const helper: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();
    const scene: Scene = new Scene();
    const drawnMesh: Mesh = new Mesh(new BufferGeometry(), surface);
    const hiddenMesh: Mesh = new Mesh(new BufferGeometry(), helper);
    const findHit = jest.fn((texel: IPickTexel): Nullable<Omit<IRendererHit, "point">> => {
      expect(texel).toEqual({ distance: 10, draw: 5, kind: EPickKind.STATIC, place: 7 });

      return { instance: 2, object: "rock", surface: "stone" };
    });
    const replies: Array<Nullable<IRendererHit>> = [];
    const picks: RendererPicks = new RendererPicks(
      { findHit, pickedScenes: [scene] } as unknown as RendererScene,
      (_: number, hit: Nullable<IRendererHit>) => replies.push(hit)
    );
    const { renderer, drawn } = createRenderer([EPickKind.STATIC, 5, 7, 10]);

    scene.add(drawnMesh, hiddenMesh);
    picks.push(1, { x: 100, y: 50 });
    picks.answer(renderer, { camera: createCamera(), size: { height: 100, pixelRatio: 1, width: 200 } });

    expect(drawn).toHaveLength(1);
    expect(drawn[0].meshes).toEqual([
      { isVisible: true, material: surface.pick },
      { isVisible: false, material: helper },
    ]);
    // The one texel under the point, of the frame's camera.
    expect((drawn[0].camera as PerspectiveCamera).view).toMatchObject({
      fullHeight: 100,
      fullWidth: 200,
      height: 1,
      offsetX: 99.5,
      offsetY: 49.5,
      width: 1,
    });
    expect(drawnMesh.material).toBe(surface);
    expect(hiddenMesh.visible).toBe(true);
    expect(renderer.setClearColor).toHaveBeenLastCalledWith(new Color(0x123456), 1);
    expect(renderer.setRenderTarget).toHaveBeenLastCalledWith(null);
    expect(picks.hasPending).toBe(false);

    await landRead();

    // Ten metres down the ray through the view's centre.
    expect(replies).toHaveLength(1);
    expect(replies[0]).toMatchObject({ instance: 2, object: "rock", surface: "stone" });
    expect(replies[0]?.point.map((it: number) => Math.round(it * 1000) / 1000)).toEqual([0, 0, -10]);
  });

  it("hits nothing where the texel was left clear, or where no object answers to it any more", async () => {
    const replies: Array<Nullable<IRendererHit>> = [];
    const findHit = jest.fn(() => null);
    const picks: RendererPicks = new RendererPicks(
      { findHit, pickedScenes: [] } as unknown as RendererScene,
      (_: number, hit: Nullable<IRendererHit>) => replies.push(hit)
    );

    picks.push(1, { x: 10, y: 10 });
    picks.answer(createRenderer([0, 0, 0, 0]).renderer, {
      camera: createCamera(),
      size: { height: 100, pixelRatio: 1, width: 200 },
    });
    picks.push(2, { x: 10, y: 10 });
    picks.answer(createRenderer([EPickKind.PLAIN, 3, 0, 1]).renderer, {
      camera: createCamera(),
      size: { height: 100, pixelRatio: 1, width: 200 },
    });
    await landRead();

    expect(replies).toEqual([null, null]);
    expect(findHit).toHaveBeenCalledTimes(1);
  });

  it("answers at once, drawing nothing, without a view or for a point outside it", () => {
    const replies: Array<[number, Nullable<IRendererHit>]> = [];
    const picks: RendererPicks = new RendererPicks({ pickedScenes: [] } as unknown as RendererScene, (id, hit) =>
      replies.push([id, hit])
    );
    const { renderer, drawn } = createRenderer([EPickKind.STATIC, 1, 1, 1]);

    picks.push(1, { x: 5, y: 5 });
    picks.answer(renderer, null);
    picks.push(2, { x: 500, y: 5 });
    picks.answer(renderer, { camera: createCamera(), size: { height: 100, pixelRatio: 1, width: 200 } });

    expect(replies).toEqual([
      [1, null],
      [2, null],
    ]);
    expect(drawn).toEqual([]);
  });

  it("answers nothing read back once it is disposed", async () => {
    const replies: Array<Nullable<IRendererHit>> = [];
    const picks: RendererPicks = new RendererPicks(
      { findHit: () => ({ instance: null, object: "a", surface: "b" }), pickedScenes: [] } as unknown as RendererScene,
      (_: number, hit: Nullable<IRendererHit>) => replies.push(hit)
    );

    picks.push(1, { x: 5, y: 5 });
    picks.answer(createRenderer([EPickKind.STATIC, 1, 1, 1]).renderer, {
      camera: createCamera(),
      size: { height: 100, pixelRatio: 1, width: 200 },
    });
    picks.dispose();
    await landRead();

    expect(replies).toEqual([]);
  });
});
