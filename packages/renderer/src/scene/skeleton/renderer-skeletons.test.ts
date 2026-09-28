import { describe, expect, it, jest } from "@jest/globals";
import { Maybe, Nullable } from "@xrf/types";

import { IRendererSkeleton } from "#/contract/scene/renderer-skeleton";
import { RendererSkeletonEntry } from "#/scene/skeleton/renderer-skeleton-entry";
import { RendererSkeletons } from "#/scene/skeleton/renderer-skeletons";

/** One bone standing at the origin, unrotated. */
function createSkeleton(): IRendererSkeleton {
  return { binds: new Float32Array([1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]) };
}

describe("RendererSkeletons", () => {
  it("keeps a skeleton put again posed and advanced until the change replacing it lets it go", () => {
    const releases: Array<Nullable<() => void>> = [];
    const skeletons: RendererSkeletons = new RendererSkeletons((_: string, release: Nullable<() => void>) =>
      releases.push(release)
    );

    skeletons.putSkeleton("npc", createSkeleton());

    const old: RendererSkeletonEntry = skeletons.get("npc") as RendererSkeletonEntry;
    const advanced = jest.spyOn(old, "advance");
    const disposed = jest.spyOn(old, "dispose");

    skeletons.putSkeleton("npc", createSkeleton());

    // Nothing replaced by the first put; the second hands over what lets the first go.
    expect(releases[0]).toBeNull();
    expect(skeletons.get("npc")).not.toBe(old);

    const version: number = old.version;

    skeletons.pose("npc", { frame: 0, hiddenBones: [0], motion: null });
    skeletons.advance();

    expect(old.version).toBe(version + 1);
    expect(advanced).toHaveBeenCalledTimes(1);
    expect(disposed).not.toHaveBeenCalled();

    (releases[1] as () => void)();
    skeletons.advance();

    expect(disposed).toHaveBeenCalledTimes(1);
    expect(advanced).toHaveBeenCalledTimes(1);
  });

  it("hands over what lets a released skeleton go, and forgets it at once", () => {
    let release: Maybe<Nullable<() => void>>;
    const skeletons: RendererSkeletons = new RendererSkeletons((_: string, it: Nullable<() => void>) => (release = it));

    skeletons.putSkeleton("npc", createSkeleton());

    const old: RendererSkeletonEntry = skeletons.get("npc") as RendererSkeletonEntry;
    const disposed = jest.spyOn(old, "dispose");

    skeletons.releaseSkeleton("npc");

    expect(skeletons.get("npc")).toBeUndefined();
    expect(disposed).not.toHaveBeenCalled();

    (release as () => void)();

    expect(disposed).toHaveBeenCalledTimes(1);
  });
});
