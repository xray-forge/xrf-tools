import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Camera, Mesh, NodeMaterial, Object3D, RenderTarget, WebGPURenderer } from "three/webgpu";

import { FullScreenDraw } from "#/pass/full-screen-draw";

interface IDrawCall {
  object: Object3D;
  camera: Camera;
  target: Nullable<RenderTarget>;
}

interface IFakeRenderer {
  renderer: WebGPURenderer;
  renders: Array<IDrawCall>;
  compiles: Array<IDrawCall>;
  finish(): Promise<void>;
  current(): Nullable<RenderTarget>;
}

function createRenderer(): IFakeRenderer {
  const renders: Array<IDrawCall> = [];
  const compiles: Array<IDrawCall> = [];
  const pending: Array<() => void> = [];
  let current: Nullable<RenderTarget> = null;

  const renderer = {
    compileAsync: (object: Object3D, camera: Camera): Promise<void> => {
      compiles.push({ camera, object, target: current });

      return new Promise((resolve: () => void) => pending.push(resolve));
    },
    getRenderTarget: (): Nullable<RenderTarget> => current,
    render: (object: Object3D, camera: Camera): void => {
      renders.push({ camera, object, target: current });
    },
    setRenderTarget: (target: Nullable<RenderTarget>): void => {
      current = target;
    },
  };

  return {
    compiles,
    current: () => current,
    finish: async (): Promise<void> => {
      pending.shift()?.();

      for (let turn: number = 0; turn < 4; turn += 1) {
        await Promise.resolve();
      }
    },
    renderer: renderer as unknown as WebGPURenderer,
    renders,
  };
}

describe("FullScreenDraw", () => {
  it("places the triangle by its vertex alone, over three corners with the uv three's quad carries", () => {
    const material: NodeMaterial = new NodeMaterial();
    const draw: FullScreenDraw = new FullScreenDraw(material, null);
    const fake: IFakeRenderer = createRenderer();

    draw.render(fake.renderer);

    const mesh: Mesh = fake.renders[0].object as Mesh;

    expect(material.vertexNode).not.toBeNull();
    expect(mesh.material).toBe(material);
    expect(mesh.geometry.getAttribute("position").count).toBe(3);
    expect(Array.from(mesh.geometry.getAttribute("uv").array)).toEqual([0, -1, 0, 1, 2, 1]);
  });

  // A compile for other attachments than the draw's builds a pipeline its draws never use.
  it("compiles what it draws, for its own target, and leaves the target that was current", () => {
    const fake: IFakeRenderer = createRenderer();
    const target: RenderTarget = new RenderTarget();
    const previous: RenderTarget = new RenderTarget();
    const draw: FullScreenDraw = new FullScreenDraw(new NodeMaterial(), target);

    fake.renderer.setRenderTarget(previous);
    void draw.compile(fake.renderer);
    draw.render(fake.renderer);

    expect(fake.compiles[0].target).toBe(target);
    expect(fake.compiles[0].object).toBe(fake.renders[0].object);
    expect(fake.compiles[0].camera).toBe(fake.renders[0].camera);
    expect(fake.renders[0].target).toBe(target);
  });

  it("draws into another target of the same attachments where asked", () => {
    const fake: IFakeRenderer = createRenderer();
    const other: RenderTarget = new RenderTarget();

    new FullScreenDraw(new NodeMaterial(), new RenderTarget()).render(fake.renderer, other);

    expect(fake.renders[0].target).toBe(other);
  });

  // Freed while three builds it, its material's bindings would be made again by the build.
  it("lets its material go once a compile in flight ends, and at once otherwise", async () => {
    const fake: IFakeRenderer = createRenderer();
    const compiling: NodeMaterial = new NodeMaterial();
    const idle: NodeMaterial = new NodeMaterial();
    const disposed: jest.Mock<() => void> = jest.fn();

    compiling.addEventListener("dispose", disposed);
    idle.addEventListener("dispose", disposed);

    const draw: FullScreenDraw = new FullScreenDraw(compiling, null);

    void draw.compile(fake.renderer);
    draw.dispose();

    expect(disposed).not.toHaveBeenCalled();

    await fake.finish();

    expect(disposed).toHaveBeenCalledTimes(1);

    new FullScreenDraw(idle, null).dispose();

    expect(disposed).toHaveBeenCalledTimes(2);
  });
});
