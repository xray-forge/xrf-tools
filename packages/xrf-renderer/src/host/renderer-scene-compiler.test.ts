import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";
import {
  Camera,
  ComputeNode,
  Mesh,
  NodeMaterial,
  Object3D,
  PerspectiveCamera,
  RenderTarget,
  Scene,
  WebGPURenderer,
} from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { ICompilingFrame } from "#/graph/compiling-frame";
import { IFrameCompileTargets } from "#/graph/frame-compile-targets";
import { FramePipelines } from "#/graph/frame-pipelines";
import { RendererSceneCompiler } from "#/host/renderer-scene-compiler";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { toPassRecord } from "#/scene/pass-record";
import { RendererScene } from "#/scene/renderer-scene";
import { ISceneBuildStaging } from "#/scene/staging/scene-build-staging";
import { ISceneStaging } from "#/scene/staging/scene-staging";

interface ICompileCall {
  /** What compiled: a scene or object, or compute kernels. */
  scene: Object3D | ReadonlyArray<ComputeNode>;
  /** The scene an object compiled in, which its materials build with. */
  into: Nullable<Scene>;
  target: Nullable<RenderTarget>;
  finish: () => void;
}

interface IFakeRenderer {
  renderer: WebGPURenderer;
  /** Every compile in the order it started, of a scene or of kernels. */
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

  function start(scene: Object3D | ReadonlyArray<ComputeNode>, into: Nullable<Scene> = null): Promise<void> {
    return new Promise((resolve: () => void) => {
      pending += 1;
      most = Math.max(most, pending);
      calls.push({
        finish: () => {
          pending -= 1;
          resolve();
        },
        into,
        scene,
        target: current,
      });
    });
  }

  const renderer = {
    _nodes: { getForCompute: (): void => {}, getForRender: (): void => {} },
    compileAsync: (scene: Object3D, _camera: Camera, into: Nullable<Scene> = null): Promise<void> => start(scene, into),
    compileComputeAsync: (kernels: ReadonlyArray<ComputeNode>): Promise<void> => start(kernels),
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
  grass: Nullable<ISceneBuildStaging>;
  rain: Nullable<ISceneBuildStaging>;
  thunder: Nullable<ISceneBuildStaging>;
}

/** Objects compiled at once, as the compiler runs them. */
const LANES: number = 8;

/**
 * @param deferred - Objects the deferred pass draws, past the lanes to keep them busy while a test changes the frame.
 */
function createStaging(deferred: number = 1): ISceneStaging {
  const scenes: ISceneStaging["scenes"] = toPassRecord(() => new Scene().add(new Mesh()));

  for (let at: number = 1; at < deferred; at += 1) {
    scenes[ERendererPass.DEFERRED].add(new Mesh());
  }

  return { materials: [], scenes, shadows: new Scene().add(new Mesh()) };
}

/** The one object a scene holds, which compiles in it. */
function only(scene: Scene): Object3D {
  return scene.children[0];
}

function createScene(grass: Nullable<ISceneBuildStaging> = null, deferred: number = 1): IFakeScene {
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
    staging: createStaging(deferred),
    thunder: null,
  };
  let isStaged: boolean = false;

  fake.scene = {
    commit: fake.commit,
    grass: {
      takeStaged: (): Nullable<ISceneBuildStaging> => {
        const taken: Nullable<ISceneBuildStaging> = fake.grass;

        fake.grass = null;

        return taken;
      },
    },
    rain: {
      takeStaged: (): Nullable<ISceneBuildStaging> => {
        const taken: Nullable<ISceneBuildStaging> = fake.rain;

        fake.rain = null;

        return taken;
      },
    },
    thunder: {
      takeStaged: (): Nullable<ISceneBuildStaging> => {
        const taken: Nullable<ISceneBuildStaging> = fake.thunder;

        fake.thunder = null;

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
    thunder: new RenderTarget(),
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
  const pipelines: FramePipelines = new FramePipelines();

  fake.frame = {
    admit: (pass: IRendererScenePass): void => {
      fake.admitted.push(pass);
    },
    get compileTargets(): IFrameCompileTargets {
      return fake.targets;
    },
    pipelines,
  };

  return fake;
}

function createStagedBuild(kernels: ReadonlyArray<ComputeNode> = []): ISceneBuildStaging {
  return { abandon: jest.fn(), commit: jest.fn(), kernels, scene: new Scene().add(new Mesh()) };
}

function toKernel(): ComputeNode {
  return { isComputeNode: true } as unknown as ComputeNode;
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
  // Three waits for each object's pipeline before the next, so a scene compiled whole made them one at a time.
  it("compiles every pass's objects side by side, each in the scene it is drawn in, against its pass's target", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, staging, commit }: IFakeScene = createScene();
    const { frame, targets }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();
    const previous: RenderTarget = new RenderTarget();

    fake.renderer.setRenderTarget(previous);
    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

    expect(fake.calls).toHaveLength(4);
    expect(fake.current()).toBe(previous);
    expect(compiler.isCompiling).toBe(true);

    await finishAll(fake);

    expect(fake.calls.map((call: ICompileCall) => call.target)).toEqual([
      ...targets.passes.map((pass: IRendererScenePass) => pass.target),
      targets.shadow.target,
    ]);
    expect(fake.calls.map((call: ICompileCall) => [call.scene, call.into])).toEqual(
      [
        staging.scenes[ERendererPass.DEFERRED],
        staging.scenes[ERendererPass.FORWARD],
        staging.scenes[ERendererPass.WATER],
        staging.shadows,
      ].map((into: Scene) => [only(into), into])
    );
    expect(commit).toHaveBeenCalledWith(
      staging,
      new Set([ERendererPass.DEFERRED, ERendererPass.FORWARD, ERendererPass.WATER])
    );
    expect(compiler.isCompiling).toBe(false);
  });

  it("compiles no more objects at once than it has lanes, each lane taking the next as it finishes one", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, staging }: IFakeScene = createScene(null, 20);
    const { frame }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

    expect(fake.calls).toHaveLength(LANES);

    await finishAll(fake);

    expect(fake.mostPending()).toBe(LANES);
    expect(fake.calls).toHaveLength(20 + 3);
    expect(fake.calls.slice(0, 20).map((call: ICompileCall) => call.scene)).toEqual(
      staging.scenes[ERendererPass.DEFERRED].children
    );
  });

  // Compiled into its freed target, a pass that left would have three allocate that target outside the frame's sizing.
  it("compiles the passes the frame holds at each step: none that left it meanwhile, and one that joined", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, staging, commit }: IFakeScene = createScene(null, LANES + 1);
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
      ...Array.from({ length: LANES + 1 }, () => deferred.target),
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

    const started: number = fake.calls.length;

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

    expect(fake.calls).toHaveLength(started);
  });

  it("compiles a grass build as a batch of its own, against where the grass draws, before the scene's", async () => {
    const fake: IFakeRenderer = createRenderer();
    const grass: ISceneBuildStaging = createStagedBuild();
    const { scene, commit }: IFakeScene = createScene(grass);
    const { frame, targets }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

    expect(fake.calls.map((call: ICompileCall) => [call.scene, call.into, call.target])).toEqual([
      [only(grass.scene), grass.scene, targets.grass],
    ]);

    await finishAll(fake);

    expect(grass.commit).toHaveBeenCalledTimes(1);
    expect(commit).not.toHaveBeenCalled();

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

    expect(fake.calls[1].target).toBe(targets.passes[0].target);
  });

  // Drawn on its first frame, the rain would build its pipelines there as the first shower starts.
  it("compiles a rain build as a batch of its own, against where the rain draws, before the scene's", async () => {
    const fake: IFakeRenderer = createRenderer();
    const rain: ISceneBuildStaging = createStagedBuild();
    const fakeScene: IFakeScene = createScene();
    const { frame, targets }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    fakeScene.rain = rain;
    compiler.compile(fake.renderer, fakeScene.scene, frame, new PerspectiveCamera());

    expect(fake.calls.map((call: ICompileCall) => [call.scene, call.target])).toEqual([
      [only(rain.scene), targets.rain],
    ]);

    await finishAll(fake);

    expect(rain.commit).toHaveBeenCalledTimes(1);
    expect(fakeScene.commit).not.toHaveBeenCalled();
  });

  // Drawn on its first strike, the bolt would build its pipelines there and hitch the flash.
  it("compiles a thunder build as a batch of its own, against where the bolts draw, before the scene's", async () => {
    const fake: IFakeRenderer = createRenderer();
    const thunder: ISceneBuildStaging = createStagedBuild();
    const fakeScene: IFakeScene = createScene();
    const { frame, targets }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    fakeScene.thunder = thunder;
    compiler.compile(fake.renderer, fakeScene.scene, frame, new PerspectiveCamera());

    expect(fake.calls.map((call: ICompileCall) => [call.scene, call.target])).toEqual([
      [only(thunder.scene), targets.thunder],
    ]);

    await finishAll(fake);

    expect(thunder.commit).toHaveBeenCalledTimes(1);
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
      [only(drawn.scenes[ERendererPass.WATER]), water.target],
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
    expect(fake.calls[0].scene).toBe(only(staging.scenes[ERendererPass.DEFERRED]));
  });

  it("compiles the water for a batch it joined during, then what was drawn before it joined", async () => {
    const fake: IFakeRenderer = createRenderer();
    const fakeScene: IFakeScene = createScene(null, LANES + 1);
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
      ...Array.from({ length: LANES + 1 }, () => deferred.target),
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

    expect(fake.calls[fake.calls.length - 1].scene).toBe(only(drawn.scenes[ERendererPass.WATER]));
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
    expect(fake.calls[calls].scene).toBe(only(fakeScene.staging.scenes[ERendererPass.DEFERRED]));
  });

  // Built as the frame draws, a pass's own pipeline is made on the thread drawing the window.
  it("compiles the frame's own pipelines side by side before anything else waits, each draw for its target", async () => {
    const fake: IFakeRenderer = createRenderer();
    const grass: ISceneBuildStaging = createStagedBuild();
    const { scene }: IFakeScene = createScene(grass);
    const { frame }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();
    const draws: Array<FullScreenDraw> = [new RenderTarget(), null].map(
      (target: Nullable<RenderTarget>) => new FullScreenDraw(new NodeMaterial(), target)
    );
    const kernels: Array<ComputeNode> = [toKernel(), toKernel()];

    frame.pipelines.draw(draws[0]);
    frame.pipelines.draw(draws[1]);
    frame.pipelines.compute(kernels);
    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

    expect(fake.calls.map((call: ICompileCall) => call.target)).toEqual([draws[0].target, null, null, null]);
    expect(fake.calls.slice(2).map((call: ICompileCall) => call.scene)).toEqual([[kernels[0]], [kernels[1]]]);

    await finishAll(fake);
    // The batch settles a few turns after its last compile.
    await settle();

    expect(fake.mostPending()).toBe(4);
    expect(frame.pipelines.isWaiting).toBe(false);
    expect(grass.commit).not.toHaveBeenCalled();

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());

    expect(fake.calls[4].scene).toBe(only(grass.scene));
  });

  // Dispatched on their first frame, the planting's passes would build their pipelines there as the grass comes in.
  it("compiles a staged build's kernels after its draws, and takes the build once both compiled", async () => {
    const fake: IFakeRenderer = createRenderer();
    const kernels: Array<ComputeNode> = [toKernel()];
    const grass: ISceneBuildStaging = createStagedBuild(kernels);
    const { scene }: IFakeScene = createScene(grass);
    const { frame }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());
    fake.calls[0].finish();
    await settle();

    expect(fake.calls.map((call: ICompileCall) => call.scene)).toEqual([only(grass.scene), kernels]);
    expect(grass.commit).not.toHaveBeenCalled();

    fake.calls[1].finish();
    await settle();

    expect(grass.commit).toHaveBeenCalledTimes(1);
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

    expect(commit).toHaveBeenCalledWith(
      staging,
      new Set([ERendererPass.DEFERRED, ERendererPass.FORWARD, ERendererPass.WATER])
    );
    expect(error).toHaveBeenCalledTimes(1);
    expect(compiler.isCompiling).toBe(false);

    error.mockRestore();
  });

  it("takes nothing it compiled once disposed, lets a grass build go, and compiles nothing more", async () => {
    const fake: IFakeRenderer = createRenderer();
    const grass: ISceneBuildStaging = createStagedBuild();
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

  it("starts no further object of a batch once disposed mid batch", async () => {
    const fake: IFakeRenderer = createRenderer();
    const { scene, commit }: IFakeScene = createScene(null, LANES + 1);
    const { frame }: IFakeFrame = createFrame();
    const compiler: RendererSceneCompiler = new RendererSceneCompiler();

    compiler.compile(fake.renderer, scene, frame, new PerspectiveCamera());
    compiler.dispose();
    await finishAll(fake);

    expect(fake.calls).toHaveLength(LANES);
    expect(commit).not.toHaveBeenCalled();
  });
});
