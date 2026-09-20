import { describe, expect, it } from "@jest/globals";

import { ILevelCamera, UNPLACED_LEVEL_CAMERA } from "@/core/level/lib/level-camera";
import { EMPTY_LEVEL_STATS, ILevelStats } from "@/core/level/lib/level-stats";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { mockInjectedService } from "@/fixtures/utils/container";

function stats(overrides: Partial<ILevelStats> = {}): ILevelStats {
  return { ...EMPTY_LEVEL_STATS, ...overrides };
}

function camera(overrides: Partial<ILevelCamera> = {}): ILevelCamera {
  return { ...UNPLACED_LEVEL_CAMERA, ...overrides };
}

describe("LevelViewportService", () => {
  it("reports nothing about a camera that has never drawn", () => {
    const { service } = mockInjectedService(LevelViewportService);

    expect(service.camera).toBeNull();
    expect(service.stats).toEqual(EMPTY_LEVEL_STATS);
  });

  it("takes what the viewport measured", () => {
    const { service } = mockInjectedService(LevelViewportService);

    service.report(stats({ draws: 493 }), camera({ heading: 1.5 }));

    expect(service.stats.draws).toBe(493);
    expect(service.camera?.heading).toBe(1.5);
  });

  // A closed viewer showing the last frame of the level before it is worse than showing nothing: it reads as a level
  // that is still open.
  it("forgets a closed level rather than keeping its last frame", () => {
    const { service } = mockInjectedService(LevelViewportService);

    service.report(stats({ draws: 493 }), camera());
    service.clear();

    expect(service.camera).toBeNull();
    expect(service.stats).toEqual(EMPTY_LEVEL_STATS);
  });

  // `report` is handed to a scene that keeps it for its lifetime, so it has to work without its receiver.
  it("reports through a bound method, since the viewport keeps the function and not the service", () => {
    const { service } = mockInjectedService(LevelViewportService);
    const report: LevelViewportService["report"] = service.report;

    report(stats({ sectors: 3 }), camera());

    expect(service.stats.sectors).toBe(3);
  });
});
