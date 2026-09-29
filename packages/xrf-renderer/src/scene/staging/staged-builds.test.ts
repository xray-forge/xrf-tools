import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Scene } from "three/webgpu";

import { ISceneBuildStaging } from "#/scene/staging/scene-build-staging";
import { StagedBuilds } from "#/scene/staging/staged-builds";

interface ITestBuild {
  name: string;
  scene: Scene;
}

function createBuild(name: string): ITestBuild {
  return { name, scene: new Scene() };
}

function createBuilds() {
  const released: Array<string> = [];
  const committed: Array<string> = [];
  const builds: StagedBuilds<ITestBuild> = new StagedBuilds({
    onCommit: (build: ITestBuild) => committed.push(build.name),
    release: (build: ITestBuild) => released.push(build.name),
  });

  return { builds, committed, released };
}

describe("StagedBuilds", () => {
  it("draws a build once it compiled, handing it over once, and lets the one before go", () => {
    const { builds, committed, released } = createBuilds();

    builds.stage(createBuild("first"));

    const first: Nullable<ISceneBuildStaging> = builds.takeStaged();

    expect(builds.takeStaged()).toBeNull();
    expect(builds.current).toBeNull();

    first?.commit();
    builds.stage(createBuild("second"));
    builds.takeStaged()?.commit();

    expect(builds.current?.name).toBe("second");
    expect(builds.pending).toBeNull();
    expect(committed).toEqual(["first", "second"]);
    expect(released).toEqual(["first"]);
  });

  it("hands an abandoned build over again", () => {
    const { builds } = createBuilds();
    const build: ITestBuild = createBuild("first");

    builds.stage(build);
    builds.takeStaged()?.abandon();

    expect(builds.waiting).toBe(build);
    expect(builds.takeStaged()?.scene).toBe(build.scene);
  });

  it("lets a build waiting go at once where another replaces it", () => {
    const { builds, released } = createBuilds();

    builds.stage(createBuild("first"));
    builds.stage(createBuild("second"));

    expect(released).toEqual(["first"]);
    expect(builds.waiting?.name).toBe("second");
  });

  // Three is still building its pipelines: taken down now, its compile would bind what is gone.
  it("lets a build go once its compile ends where it is let go while compiling", () => {
    const { builds, committed, released } = createBuilds();

    builds.stage(createBuild("first"));

    const first: Nullable<ISceneBuildStaging> = builds.takeStaged();

    builds.stage(createBuild("second"));

    expect(released).toEqual([]);
    expect(builds.compiling?.name).toBe("first");
    // One compile at a time: the next waits for the one in flight.
    expect(builds.takeStaged()).toBeNull();

    first?.commit();

    expect(released).toEqual(["first"]);
    expect(committed).toEqual([]);
    expect(builds.current).toBeNull();
    expect(builds.takeStaged()?.scene).toBe(builds.pending?.scene);
  });

  it("lets every build go when cleared, the one compiling once it ends", () => {
    const { builds, released } = createBuilds();

    builds.stage(createBuild("first"));
    builds.takeStaged()?.commit();
    builds.stage(createBuild("second"));

    const second: Nullable<ISceneBuildStaging> = builds.takeStaged();

    builds.clear();

    expect(released).toEqual(["first"]);

    second?.abandon();

    expect(released).toEqual(["first", "second"]);
    expect(builds.current).toBeNull();
    expect(builds.pending).toBeNull();
  });
});
