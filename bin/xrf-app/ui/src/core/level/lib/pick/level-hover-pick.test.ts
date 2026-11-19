import { describe, expect, it, jest } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { LatestFlow, TFlow } from "@xrf/flow";
import { ERenderLevelHit, RenderLevelHit } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { LevelHoverPick } from "@/core/level/lib/pick/level-hover-pick";
import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";

const HIT: RenderLevelHit = { depth: 1, kind: ERenderLevelHit.SPAWN, object: 3, point: [0, 0, 0] };

// Each point takes over the last, as a service's hover does.
class Pointer {
  public hovered: Array<Nullable<RenderLevelHit>> = [];

  public readonly picks: LevelHoverPick = new LevelHoverPick();

  public constructor(public viewport: Nullable<NativeViewport>) {}

  @LatestFlow()
  public *hover(point: IRenderViewPoint): TFlow {
    this.hovered.push(yield* this.picks.pick(point, () => this.viewport));
  }
}

function mockViewport(): { viewport: NativeViewport; answers: Array<(hit: RenderLevelHit) => void>; pick: jest.Mock } {
  const answers: Array<(hit: RenderLevelHit) => void> = [];
  const pick = jest.fn(
    () => new Promise<RenderLevelHit>((resolve: (hit: RenderLevelHit) => void) => answers.push(resolve))
  );

  return { answers, pick, viewport: { pick } as unknown as NativeViewport };
}

describe("LevelHoverPick", () => {
  it("keeps one pick in flight, picking the latest point waiting once it settles", async () => {
    const { answers, pick, viewport } = mockViewport();
    const pointer: Pointer = new Pointer(viewport);

    const first = pointer.hover({ x: 1, y: 1 });
    const second = pointer.hover({ x: 2, y: 2 });
    const third = pointer.hover({ x: 3, y: 3 });

    await waitFor(() => expect(answers).toHaveLength(1));
    answers[0]?.(HIT);
    await waitFor(() => expect(answers).toHaveLength(2));
    answers[1]?.(HIT);
    await Promise.allSettled([first, second, third]);

    expect(pick.mock.calls).toEqual([[{ isThrough: false, x: 1, y: 1 }], [{ isThrough: false, x: 3, y: 3 }]]);
    expect(pointer.hovered).toEqual([HIT]);
  });

  it("answers nothing where the viewport went while the pick in flight settled", async () => {
    const { answers, pick, viewport } = mockViewport();
    const pointer: Pointer = new Pointer(viewport);

    const first = pointer.hover({ x: 1, y: 1 });

    await waitFor(() => expect(answers).toHaveLength(1));

    const second = pointer.hover({ x: 2, y: 2 });

    pointer.viewport = null;
    answers[0]?.(HIT);
    await Promise.allSettled([first, second]);

    expect(pick).toHaveBeenCalledTimes(1);
    expect(pointer.hovered).toEqual([null]);
  });

  it("picks at once after a reset, the pick in flight forgotten", async () => {
    const { answers, pick, viewport } = mockViewport();
    const pointer: Pointer = new Pointer(viewport);

    const first = pointer.hover({ x: 1, y: 1 });

    await waitFor(() => expect(answers).toHaveLength(1));

    pointer.picks.reset();

    const next = pointer.hover({ x: 2, y: 2 });

    await waitFor(() => expect(answers).toHaveLength(2));
    answers[1]?.(HIT);
    await Promise.allSettled([first, next]);

    expect(pick).toHaveBeenCalledTimes(2);
  });
});
