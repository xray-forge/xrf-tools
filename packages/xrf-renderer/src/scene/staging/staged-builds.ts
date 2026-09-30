import { Nullable } from "@xrf/types";
import { ComputeNode, Scene } from "three/webgpu";

import { ISceneBuildStaging } from "#/scene/staging/scene-build-staging";
import { IStagedBuildsInput } from "#/scene/staging/staged-builds-input";

/**
 * The build a scene draws and the one built to replace it, which the renderer compiles off the frame first, so no
 * frame builds a draw's pipeline. One build waits at a time, and one is compiled at a time. A build let go while it
 * compiles is taken down once its compile ends, since three is still building its pipelines until then.
 */
export class StagedBuilds<TBuild extends { scene: Scene; kernels?: ReadonlyArray<ComputeNode> }> {
  private readonly release: (build: TBuild) => void;
  private readonly onCommit: (build: TBuild) => void;
  private drawing: Nullable<TBuild> = null;
  /** Built to replace what draws, waiting to compile or compiling. */
  private next: Nullable<TBuild> = null;
  /** The build the renderer took to compile, which may since have been let go. */
  private compiled: Nullable<TBuild> = null;

  public constructor(input: IStagedBuildsInput<TBuild>) {
    this.release = input.release;
    this.onCommit = input.onCommit ?? ((): void => {});
  }

  /** What draws, or null before any build compiled. */
  public get current(): Nullable<TBuild> {
    return this.drawing;
  }

  /** What is built to replace it, compiling or not, or null for none. */
  public get pending(): Nullable<TBuild> {
    return this.next;
  }

  /** The build to replace what draws while the renderer has not taken it to compile, or null. */
  public get waiting(): Nullable<TBuild> {
    return this.next !== this.compiled ? this.next : null;
  }

  /** The build compiling, whether it still replaces what draws or was let go meanwhile, or null. */
  public get compiling(): Nullable<TBuild> {
    return this.compiled;
  }

  /**
   * Puts a build up to replace what draws once it compiled, letting the one waiting before it go.
   *
   * @param build - The build, or null to replace what draws with nothing new.
   */
  public stage(build: Nullable<TBuild>): void {
    this.drop(this.next);
    this.next = build;
  }

  /** Lets every build go. */
  public clear(): void {
    this.drop(this.drawing);
    this.drop(this.next);
    this.drawing = null;
    this.next = null;
  }

  /**
   * @returns The build waiting to compile, handed over once, or null while none waits or one compiles.
   */
  public takeStaged(): Nullable<ISceneBuildStaging> {
    const build: Nullable<TBuild> = this.next;

    if (!build || this.compiled) {
      return null;
    }

    this.compiled = build;

    return {
      abandon: (): void => this.settle(build, false),
      commit: (): void => this.settle(build, true),
      kernels: build.kernels ?? [],
      scene: build.scene,
    };
  }

  /** Lets a build go now, or once its compile ends where it compiles. */
  private drop(build: Nullable<TBuild>): void {
    if (build && build !== this.compiled) {
      this.release(build);
    }
  }

  private settle(build: TBuild, isCompiled: boolean): void {
    this.compiled = null;

    // Let go while it compiled: three is done with it now.
    if (build !== this.next) {
      this.release(build);

      return;
    }

    if (isCompiled) {
      this.drop(this.drawing);
      this.drawing = build;
      this.next = null;
      this.onCommit(build);
    }
  }
}
