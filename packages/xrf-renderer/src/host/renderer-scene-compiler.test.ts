import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Camera, Mesh, Object3D, PerspectiveCamera, RenderTarget, Scene, WebGPURenderer } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { ICompilingFrame } from "#/graph/compiling-frame";
import { IFrameCompileTargets } from "#/graph/frame-compile-targets";
import { RendererSceneCompiler } from "#/host/renderer-scene-compiler";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { ISceneGrassStaging } from "#/scene/grass/scene-grass-staging";
import { toPassRecord } from "#/scene/pass-record";
import { ISceneRainStaging } from "#/scene/rain/scene-rain-staging";
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

type TCommit = (staging: object, passes: ReadonlySet<ERendererPass>) => void;

interface IFakeScene {
  scene: RendererScene;
  staging: ISceneStaging;
  drawn: Nullable<ISceneStaging>;
  commit: jest.Mock<TCommit>;
  grass: Nullable<ISceneGrassStaging>;
  rain: Nullable<ISceneRainStaging>;
}

function createStaging(): ISceneStaging {
  return {
    materials: [],
    scenes: toPassRecord(() => new Scene().add(new Mesh())),
    shadows: new Scene().add(new Mesh()),
  };
}

function createScene(grass: Nullable<ISceneGrassStaging> = null): IFakeScene {
  const fake: IFakeScene = {
    commit: jest.fn((staging: object) => {
      if (staging === fake.drawn) {
        fake.drawn = null;
      }
    }),
    drawn: null,
    grass,
    rain: null,
    scene: null as unknown as RendererScene,
    staging: createStaging(),
  };
  let isStaged: boolean = false;

  fake.scene = {
    commit: fake.commit,
    grass: {
      takeStaged: (): Nullable<ISceneGrassStaging> => {
        const taken: Nullable<ISceneGrassStaging> = fake.grass;

        fake.grass = null;

        return taken;
      },
    },
    rain: {
      takeStaged: (): Nullable<ISceneRainStaging> => {
        const taken: Nullable<ISceneRainStaging> = fake.rain;

        fake.rain = null;

        return taken;
      },
    },
    get hasPending(): boolean {
      return !isStaged;
    },
    stage: (): Nullable<ISceneStaging> => {
      isStaged = true;

      return fake.staging;
    },
    stageDrawn: (): Nullable<ISceneStaging> => fake.drawn,
  } as unknown as RendererScene;

  return fake;
}

function toPass(scene: ERendererPass): IRendererScenePass {
  return { dispose: () => {}, isScenePass: true, name: scene, render: () => {}, scene, target: new RenderTarget() };
}

function createTargets(): IFrameCompileTargets {
  return {
    grass: new RenderTarget(),
    joining: [],
    rain: new RenderTarget(),
    passes: [toPass(ERendererPass.DEFERRED), toPass(ERendererPass.FORWARD), toPass(ERendererPass.WATER)],
    shadow: { camera: new PerspectiveCamera(), target: new RenderTarget() },
  };
}

interface IFakeFrame {
  frame: ICompilingFrame;
  /** What the frame holds now, which a test changes as a configure would. */
  targets: IFrameCompileTargets;
  admitted: Array<IRendererScenePass>;
}

function createFrame(targets: IFrameCompileTargets = createTargets()): IFakeFrame {
  const fake: IFakeFrame = { admitted: [], frame: null as unknown as ICompilingFrame, targets };

  fake.frame = {
    admit: (pass: IRendererScenePass): void => {
      fake.admitted.push(pass);
    },
    get compileTargets(): IFrameCompileTargets {
      return fake.targets;
    },
  };

  return fake;
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
    const { frame, targets }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();
    const previous: RenderTarget = new RenderTarget();

    fake.renderer.setRenderTarget(previous);
    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

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
    expect(commit).toHaveBeenCalledWith(
      staging,
      new Set([ERendererPass.DEFERRED, ERendererPass.FORWARD, ERendererPass.WATER])
    );
    expect(compiler.isCompiling).toBe(false);
  });

  // Compiled into its freed target, a pass that left would have three allocate that target outside the frame's sizing.
  it("compiles the passes the frame holds at each step: none that left it meanwhile, and one that joined", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, staging, commit }: IFakeScene = createScene();
    const [deferred, wallmarks, forward, water] = [
      ERendererPass.DEFERRED,
      ERendererPass.WALLMARK,
      ERendererPass.FORWARD,
      ERendererPass.WATER,
    ].map(toPass);
    const fakeFrame: IFakeFrame = createFrame({ ...createTargets(), passes: [deferred, forward, water] });
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, fakeFrame.frame, new PerspectiveCamera());
    // The water leaves the frame, and the wall marks join it, while the first pass compiles.
    fakeFrame.targets = { ...fakeFrame.targets, passes: [deferred, wallmarks, forward] };
    await finishAll(fake);

    expect(fake.calls.map((call: ICompileCall) => call.target)).toEqual([
      deferred.target,
      wallmarks.target,
      forward.target,
      fakeFrame.targets.shadow.target,
    ]);
    // What the water would have drawn is still to compile for it.
    expect(commit).toHaveBeenCalledTimes(1);
    expect(commit).toHaveBeenCalledWith(
      staging,
      new Set([ERendererPass.DEFERRED, ERendererPass.WALLMARK, ERendererPass.FORWARD])
    );
  });

  it("starts nothing while a batch compiles", () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene }: IFakeScene = createScene();
    const { frame }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());
    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

    expect(fake.calls).toHaveLength(1);
  });

  it("compiles a grass build as a batch of its own, against where the grass draws, before the scene's", async () => {
    const fake: IFakeRenderer = createRenderer();
    const grass: ISceneGrassStaging = { abandon: jest.fn(), commit: jest.fn(), scene: new Scene() };
    const { scene, commit }: IFakeScene = createScene(grass);
    const { frame, targets }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

    expect(fake.calls.map((call: ICompileCall) => [call.scene, call.target])).toEqual([[grass.scene, targets.grass]]);

    await finishAll(fake);

    expect(grass.commit).toHaveBeenCalledTimes(1);
    expect(commit).not.toHaveBeenCalled();

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

    expect(fake.calls).toHaveLength(2);
    expect(fake.calls[1].target).toBe(targets.passes[0].target);
  });

  // Drawn on its first frame, the rain would build its pipelines there as the first shower starts.
  it("compiles a rain build as a batch of its own, against where the rain draws, before the scene's", async () => {
    const fake: IFakeRenderer = createRenderer();
    const rain: ISceneRainStaging = { abandon: jest.fn(), commit: jest.fn(), scene: new Scene() };
    const fakeScene: IFakeScene = createScene();
    const { frame, targets }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    fakeScene.rain = rain;
    compiler.compile(fake.renderer, fakeScene.scene, frame, new PerspectiveCamera());

    expect(fake.calls.map((call: ICompileCall) => [call.scene, call.target])).toEqual([[rain.scene, targets.rain]]);

    await finishAll(fake);

    expect(rain.commit).toHaveBeenCalledTimes(1);
    expect(fakeScene.commit).not.toHaveBeenCalled();
  });

  // Drawn on its first frame, the water's surfaces would build their pipelines there, freezing the whole window.
  it("compiles a pass joining the frame for what the scene draws already, and admits it only then", async () => {
    const fake: IFakeRenderer = createRenderer();
    const fakeScene: IFakeScene = createScene();
    const drawn: ISceneStaging = createStaging();
    const water: IRendererScenePass = toPass(ERendererPass.WATER);
    const fakeFrame: IFakeFrame = createFrame({ ...createTargets(), joining: [water] });
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    fakeScene.drawn = drawn;
    compiler.compile(fake.renderer, fakeScene.scene, fakeFrame.frame, new PerspectiveCamera());

    expect(fake.calls.map((call: ICompileCall) => [call.scene, call.target])).toEqual([
      [drawn.scenes[ERendererPass.WATER], water.target],
    ]);
    expect(fakeFrame.admitted).toEqual([]);

    await finishAll(fake);

    expect(fakeScene.commit).toHaveBeenCalledWith(drawn, new Set([ERendererPass.WATER]));
    expect(fakeFrame.admitted).toEqual([water]);
    // The waiting objects come after.
    expect(fake.calls).toHaveLength(1);
  });

  // Switched off and on, the water's pipelines are still three's: it joins without a compile.
  it("admits a pass joining with nothing drawn to compile for it at once, and goes on to the waiting objects", () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, staging }: IFakeScene = createScene();
    const water: IRendererScenePass = toPass(ERendererPass.WATER);
    const fakeFrame: IFakeFrame = createFrame({ ...createTargets(), joining: [water] });
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, fakeFrame.frame, new PerspectiveCamera());

    expect(fakeFrame.admitted).toEqual([water]);
    expect(fake.calls.map((call: ICompileCall) => call.scene)).toEqual([staging.scenes[ERendererPass.DEFERRED]]);
  });

  it("compiles the water for a batch it joined during, then what was drawn before it joined", async () => {
    const fake: IFakeRenderer = createRenderer();
    const fakeScene: IFakeScene = createScene();
    const [deferred, forward, water] = [ERendererPass.DEFERRED, ERendererPass.FORWARD, ERendererPass.WATER].map(toPass);
    const fakeFrame: IFakeFrame = createFrame({ ...createTargets(), passes: [deferred, forward] });
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();
    const drawn: ISceneStaging = createStaging();

    compiler.compile(fake.renderer, fakeScene.scene, fakeFrame.frame, new PerspectiveCamera());
    // Switched on while the waiting objects compile.
    fakeFrame.targets = { ...fakeFrame.targets, joining: [water], passes: [deferred, water, forward] };
    fakeScene.drawn = drawn;
    await finishAll(fake);

    expect(fake.calls.map((call: ICompileCall) => call.target)).toEqual([
      deferred.target,
      water.target,
      forward.target,
      fakeFrame.targets.shadow.target,
    ]);
    expect(fakeScene.commit).toHaveBeenLastCalledWith(
      fakeScene.staging,
      new Set([ERendererPass.DEFERRED, ERendererPass.WATER, ERendererPass.FORWARD])
    );
    expect(fakeFrame.admitted).toEqual([]);

    compiler.compile(fake.renderer, fakeScene.scene, fakeFrame.frame, new PerspectiveCamera());
    await finishAll(fake);

    expect(fake.calls[fake.calls.length - 1].scene).toBe(drawn.scenes[ERendererPass.WATER]);
    expect(fakeFrame.admitted).toEqual([water]);
  });

  // The frame admits none but a pass still joining; what compiled for the old one stays compiled for the new.
  it("takes a join that the water left or was made again during, and compiles nothing for the new one", async () => {
    const fake: IFakeRenderer = createRenderer();
    const fakeScene: IFakeScene = createScene();
    const [first, second] = [ERendererPass.WATER, ERendererPass.WATER].map(toPass);
    const fakeFrame: IFakeFrame = createFrame({ ...createTargets(), joining: [first] });
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();
    const drawn: ISceneStaging = createStaging();

    fakeScene.drawn = drawn;
    compiler.compile(fake.renderer, fakeScene.scene, fakeFrame.frame, new PerspectiveCamera());
    // Off, then on again, while its join compiles.
    fakeFrame.targets = { ...fakeFrame.targets, joining: [second] };
    await finishAll(fake);

    expect(fakeScene.commit).toHaveBeenCalledWith(drawn, new Set([ERendererPass.WATER]));
    expect(fakeFrame.admitted).toEqual([first]);

    const calls: number = fake.calls.length;

    compiler.compile(fake.renderer, fakeScene.scene, fakeFrame.frame, new PerspectiveCamera());

    expect(fakeFrame.admitted).toEqual([first, second]);
    expect(fake.calls[calls].scene).toBe(fakeScene.staging.scenes[ERendererPass.DEFERRED]);
  });

  // Staged again, a batch that failed would fail the same way every frame, and nothing would ever settle.
  it("takes a batch that failed to compile as compiled for the passes it tried, as three makes of its materials", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, staging, commit }: IFakeScene = createScene();
    const { frame }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();
    const error: jest.SpiedFunction<typeof console.error> = jest.spyOn(console, "error").mockImplementation(() => {});

    (fake.renderer as unknown as { compileAsync: () => Promise<void> }).compileAsync = () =>
      Promise.reject(new Error("refused"));
    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());
    await settle();

    expect(commit).toHaveBeenCalledWith(staging, new Set([ERendererPass.DEFERRED]));
    expect(error).toHaveBeenCalledTimes(1);
    expect(compiler.isCompiling).toBe(false);

    error.mockRestore();
  });

  it("takes nothing it compiled once disposed, lets a grass build go, and compiles nothing more", async () => {
    const fake: IFakeRenderer = createRenderer();
    const grass: ISceneGrassStaging = { abandon: jest.fn(), commit: jest.fn(), scene: new Scene() };
    const first: IFakeScene = createScene(grass);
    const second: IFakeScene = createScene();
    const { frame }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, first.scene, frame, new PerspectiveCamera());
    compiler.dispose();
    await finishAll(fake);

    expect(grass.commit).not.toHaveBeenCalled();
    expect(grass.abandon).toHaveBeenCalledTimes(1);

    compiler.compile(fake.renderer, second.scene, frame, new PerspectiveCamera());

    expect(fake.calls).toHaveLength(1);
    expect(second.commit).not.toHaveBeenCalled();
  });

  it("admits no pass whose join compiled once disposed", async () => {
    const fake: IFakeRenderer = createRenderer();
    const fakeScene: IFakeScene = createScene();
    const fakeFrame: IFakeFrame = createFrame({ ...createTargets(), joining: [toPass(ERendererPass.WATER)] });
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    fakeScene.drawn = createStaging();
    compiler.compile(fake.renderer, fakeScene.scene, fakeFrame.frame, new PerspectiveCamera());
    compiler.dispose();
    await finishAll(fake);

    expect(fakeScene.commit).not.toHaveBeenCalled();
    expect(fakeFrame.admitted).toEqual([]);
  });

  it("logs nothing of a batch that fails once dropped, as the renderer goes", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, commit }: IFakeScene = createScene();
    const { frame }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();
    const error: jest.SpiedFunction<typeof console.error> = jest.spyOn(console, "error").mockImplementation(() => {});
    const compiling: { reject: (error: Error) => void } = { reject: () => {} };

    (fake.renderer as unknown as { compileAsync: () => Promise<void> }).compileAsync = () =>
      new Promise((_: () => void, reject: (error: Error) => void) => {
        compiling.reject = reject;
      });
    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());
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
    const { frame }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());
    compiler.dispose();
    await finishAll(fake);

    expect(fake.calls).toHaveLength(1);
    expect(commit).not.toHaveBeenCalled();
  });
});
