import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Camera, Mesh, Object3D, PerspectiveCamera, RenderTarget, Scene, WebGPURenderer } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { IFrameCompileTargets } from "#/graph/frame-compile-targets";
import { RendererSceneCompiler } from "#/host/renderer-scene-compiler";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { ISceneGrassStaging } from "#/scene/grass/scene-grass-staging";
import { toPassRecord } from "#/scene/pass-record";
import { RendererScene } from "#/scene/renderer-scene";
import { ISceneStaging } from "#/scene/staging/scene-staging";

interface ICompileCall {
  scene: Object3D;
  target: Nullable<RenderTarget>;
  finish: () => void;
}

interface IFakeRenderer {
  renderer: WebGPURenderer;
  calls: Array<ICompileCall>;
  /** The most compiles ever pending together. */
  mostPending(): number;
  current(): Nullable<RenderTarget>;
}

function createRenderer(): IFakeRenderer {
  const calls: Array<ICompileCall> = [];
  let current: Nullable<RenderTarget> = null;
  let pending: number = 0;
  let most: number = 0;

  const renderer = {
    compileAsync: (scene: Object3D, _camera: Camera): Promise<void> =>
      new Promise((resolve: () => void) => {
        pending += 1;
        most = Math.max(most, pending);
        calls.push({
          finish: () => {
            pending -= 1;
            resolve();
          },
          scene,
          target: current,
        });
      }),
    getRenderTarget: (): Nullable<RenderTarget> => current,
    setRenderTarget: (target: Nullable<RenderTarget>): void => {
      current = target;
    },
  };

  return {
    calls,
    current: () => current,
    mostPending: () => most,
    renderer: renderer as unknown as WebGPURenderer,
  };
}

interface IFakeScene {
  scene: RendererScene;
  staging: ISceneStaging;
  commit: jest.Mock<(staging: object) => void>;
  grass: Nullable<ISceneGrassStaging>;
}

function createScene(grass: Nullable<ISceneGrassStaging> = null): IFakeScene {
  const staging: ISceneStaging = {
    materials: [],
    scenes: toPassRecord(() => new Scene()),
    shadows: new Scene().add(new Mesh()),
  };
  const commit: jest.Mock<(staging: object) => void> = jest.fn();
  const fake: IFakeScene = { commit, grass, scene: null as unknown as RendererScene, staging };
  let isStaged: boolean = false;

  fake.scene = {
    commit,
    grass: {
      takeStaged: (): Nullable<ISceneGrassStaging> => {
        const taken: Nullable<ISceneGrassStaging> = fake.grass;

        fake.grass = null;

        return taken;
      },
    },
    get hasPending(): boolean {
      return !isStaged;
    },
    stage: (): Nullable<ISceneStaging> => {
      isStaged = true;

      return staging;
    },
  } as unknown as RendererScene;

  return fake;
}

function toPass(scene: ERendererPass): IRendererScenePass {
  return { dispose: () => {}, isScenePass: true, name: scene, render: () => {}, scene, target: new RenderTarget() };
}

function createTargets(): IFrameCompileTargets {
  return {
    grass: new RenderTarget(),
    passes: [toPass(ERendererPass.DEFERRED), toPass(ERendererPass.FORWARD), toPass(ERendererPass.WATER)],
    shadow: { camera: new PerspectiveCamera(), target: new RenderTarget() },
  };
}

function toGetter(targets: IFrameCompileTargets): () => IFrameCompileTargets {
  return () => targets;
}

async function settle(): Promise<void> {
  for (let turn: number = 0; turn < 8; turn += 1) {
    await Promise.resolve();
  }
}

async function finishAll(fake: IFakeRenderer): Promise<void> {
  for (let at: number = 0; at < fake.calls.length; at += 1) {
    fake.calls[at].finish();
    await settle();
  }
}

describe("RendererSceneCompiler", () => {
  // Two of three's asynchronous builds interleaved corrupt its bind groups.
  it("compiles one pass after another, never two at once, each against its own target", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, staging, commit }: IFakeScene = createScene();
    const targets: IFrameCompileTargets = createTargets();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();
    const previous: RenderTarget = new RenderTarget();

    fake.renderer.setRenderTarget(previous);
    compiler.compile(fake.renderer, scene, toGetter(targets), new PerspectiveCamera());

    expect(fake.calls).toHaveLength(1);
    expect(fake.current()).toBe(previous);
    expect(compiler.isCompiling).toBe(true);

    await finishAll(fake);

    expect(fake.mostPending()).toBe(1);
    expect(fake.calls.map((call: ICompileCall) => call.target)).toEqual([
      ...targets.passes.map((pass: IRendererScenePass) => pass.target),
      targets.shadow.target,
    ]);
    expect(fake.calls.map((call: ICompileCall) => call.scene)).toEqual([
      staging.scenes[ERendererPass.DEFERRED],
      staging.scenes[ERendererPass.FORWARD],
      staging.scenes[ERendererPass.WATER],
      staging.shadows,
    ]);
    expect(commit).toHaveBeenCalledWith(staging);
    expect(compiler.isCompiling).toBe(false);
  });

  // Compiled into its freed target, a pass that left would have three allocate that target outside the frame's sizing.
  it("compiles the passes the frame holds at each step: none that left it meanwhile, and one that joined", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, commit }: IFakeScene = createScene();
    const [deferred, wallmarks, forward, water] = [
      ERendererPass.DEFERRED,
      ERendererPass.WALLMARK,
      ERendererPass.FORWARD,
      ERendererPass.WATER,
    ].map(toPass);
    let targets: IFrameCompileTargets = { ...createTargets(), passes: [deferred, forward, water] };
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, () => targets, new PerspectiveCamera());
    // The water leaves the frame, and the wall marks join it, while the first pass compiles.
    targets = { ...targets, passes: [deferred, wallmarks, forward] };
    await finishAll(fake);

    expect(fake.calls.map((call: ICompileCall) => call.target)).toEqual([
      deferred.target,
      wallmarks.target,
      forward.target,
      targets.shadow.target,
    ]);
    expect(commit).toHaveBeenCalledTimes(1);
  });

  it("starts nothing while a batch compiles", () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene }: IFakeScene = createScene();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, toGetter(createTargets()), new PerspectiveCamera());
    compiler.compile(fake.renderer, scene, toGetter(createTargets()), new PerspectiveCamera());

    expect(fake.calls).toHaveLength(1);
  });

  it("compiles a grass build as a batch of its own, against where the grass draws, before the scene's", async () => {
    const fake: IFakeRenderer = createRenderer();
    const grass: ISceneGrassStaging = { abandon: jest.fn(), commit: jest.fn(), scene: new Scene() };
    const { scene, commit }: IFakeScene = createScene(grass);
    const targets: IFrameCompileTargets = createTargets();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, toGetter(targets), new PerspectiveCamera());

    expect(fake.calls.map((call: ICompileCall) => [call.scene, call.target])).toEqual([[grass.scene, targets.grass]]);

    await finishAll(fake);

    expect(grass.commit).toHaveBeenCalledTimes(1);
    expect(commit).not.toHaveBeenCalled();

    compiler.compile(fake.renderer, scene, toGetter(targets), new PerspectiveCamera());

    expect(fake.calls).toHaveLength(2);
    expect(fake.calls[1].target).toBe(targets.passes[0].target);
  });

  // Staged again, a batch that failed would fail the same way every frame, and nothing would ever settle.
  it("takes a batch that failed to compile, as three makes of its materials", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, staging, commit }: IFakeScene = createScene();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();
    const error: jest.SpiedFunction<typeof console.error> = jest.spyOn(console, "error").mockImplementation(() => {});

    (fake.renderer as unknown as { compileAsync: () => Promise<void> }).compileAsync = () =>
      Promise.reject(new Error("refused"));
    compiler.compile(fake.renderer, scene, toGetter(createTargets()), new PerspectiveCamera());
    await settle();

    expect(commit).toHaveBeenCalledWith(staging);
    expect(error).toHaveBeenCalledTimes(1);
    expect(compiler.isCompiling).toBe(false);

    error.mockRestore();
  });

  it("takes nothing it compiled once disposed, lets a grass build go, and compiles nothing more", async () => {
    const fake: IFakeRenderer = createRenderer();
    const grass: ISceneGrassStaging = { abandon: jest.fn(), commit: jest.fn(), scene: new Scene() };
    const first: IFakeScene = createScene(grass);
    const second: IFakeScene = createScene();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, first.scene, toGetter(createTargets()), new PerspectiveCamera());
    compiler.dispose();
    await finishAll(fake);

    expect(grass.commit).not.toHaveBeenCalled();
    expect(grass.abandon).toHaveBeenCalledTimes(1);

    compiler.compile(fake.renderer, second.scene, toGetter(createTargets()), new PerspectiveCamera());

    expect(fake.calls).toHaveLength(1);
    expect(second.commit).not.toHaveBeenCalled();
  });

  it("logs nothing of a batch that fails once dropped, as the renderer goes", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, commit }: IFakeScene = createScene();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();
    const error: jest.SpiedFunction<typeof console.error> = jest.spyOn(console, "error").mockImplementation(() => {});
    const compiling: { reject: (error: Error) => void } = { reject: () => {} };

    (fake.renderer as unknown as { compileAsync: () => Promise<void> }).compileAsync = () =>
      new Promise((_: () => void, reject: (error: Error) => void) => {
        compiling.reject = reject;
      });
    compiler.compile(fake.renderer, scene, toGetter(createTargets()), new PerspectiveCamera());
    compiler.dispose();
    compiling.reject(new Error("the device was destroyed"));
    await settle();

    expect(error).not.toHaveBeenCalled();
    expect(commit).not.toHaveBeenCalled();

    error.mockRestore();
  });

  it("starts no further pass of a batch once disposed mid batch", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, commit }: IFakeScene = createScene();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, toGetter(createTargets()), new PerspectiveCamera());
    compiler.dispose();
    await finishAll(fake);

    expect(fake.calls).toHaveLength(1);
    expect(commit).not.toHaveBeenCalled();
  });
});
