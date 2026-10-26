import { describe, expect, it } from "@jest/globals";

import {
  compareLevelViewerSnapshot,
  ELevelViewerSnapshotStatus,
  ILevelViewerSnapshot,
  ILevelViewerSnapshotEntry,
  LEVEL_VIEWER_SNAPSHOT_FORMAT,
  parseLevelViewerSnapshot,
  serializeLevelViewerSnapshot,
} from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { DEFAULT_RENDER_EXPOSURE_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";

const CURRENT: ILevelViewerSnapshot = {
  camera: { fieldOfView: 80, heading: 0, pitch: 0, x: 1, y: 0, z: 0 },
  format: LEVEL_VIEWER_SNAPSHOT_FORMAT,
  level: "levels/zaton",
  options: { isGrassy: true },
  output: { scale: 1 },
  settings: { antialiasing: "taa", reflections: { steps: 96 }, shadows: { swayRate: 60 }, textureFiltering: "x" },
  weather: { time: 3600 },
};

/**
 * @param file - What the file holds besides its format.
 * @param level - The level open.
 * @returns Each value of the file by its key, compared.
 */
function compare(
  file: Omit<ILevelViewerSnapshot, "format">,
  level: string = "levels/zaton"
): Record<string, ILevelViewerSnapshotEntry> {
  const entries: Array<ILevelViewerSnapshotEntry> = compareLevelViewerSnapshot(
    { format: LEVEL_VIEWER_SNAPSHOT_FORMAT, ...file },
    CURRENT,
    { exposure: DEFAULT_RENDER_EXPOSURE_SETTINGS, level }
  );

  return Object.fromEntries(entries.map((it: ILevelViewerSnapshotEntry) => [it.key, it]));
}

describe("level view snapshot", () => {
  it("reads back what it writes, and refuses what is not a level view", () => {
    const snapshot: ILevelViewerSnapshot = {
      format: LEVEL_VIEWER_SNAPSHOT_FORMAT,
      settings: { reflections: { steps: 48 } },
    };

    expect(parseLevelViewerSnapshot(serializeLevelViewerSnapshot(snapshot))).toEqual(snapshot);
    expect(() => parseLevelViewerSnapshot("{not json")).toThrow(/not JSON/);
    expect(() => parseLevelViewerSnapshot(JSON.stringify({ format: "other" }))).toThrow(/not a level view/);
  });

  it("changes what differs, as the file has it or as this build bounds it, and passes over what is the same", () => {
    const entries: Record<string, ILevelViewerSnapshotEntry> = compare({
      camera: { fieldOfView: 80 },
      options: { isGrassy: false },
      settings: { reflections: { steps: 48 }, shadows: { swayRate: 10 } },
      weather: { time: 90_000 },
    });

    expect(entries["settings.reflections.steps"]).toMatchObject({
      applied: 48,
      status: ELevelViewerSnapshotStatus.CHANGE,
    });
    expect(entries["settings.shadows.swayRate"]).toMatchObject({
      applied: 15,
      status: ELevelViewerSnapshotStatus.CHANGE,
    });
    expect(entries["weather.time"]).toMatchObject({ applied: 3600, status: ELevelViewerSnapshotStatus.SAME });
    expect(entries["options.isGrassy"].status).toBe(ELevelViewerSnapshotStatus.CHANGE);
    expect(entries["camera.fieldOfView"].status).toBe(ELevelViewerSnapshotStatus.SAME);
  });

  it("skips what the view does not take, unless the view draws it so already", () => {
    const entries: Record<string, ILevelViewerSnapshotEntry> = compare({
      options: { isGone: true },
      settings: { antialiasing: "none", reflections: { quality: "medium" }, textureFiltering: "x" },
      unknown: 1,
    } as Omit<ILevelViewerSnapshot, "format">);

    for (const key of ["options.isGone", "settings.antialiasing", "settings.reflections.quality", "unknown"]) {
      expect(entries[key]).toMatchObject({ applied: undefined, status: ELevelViewerSnapshotStatus.SKIP });
    }

    expect(entries["settings.reflections.quality"].reason).toBe("not a setting the view sets for itself");
    expect(entries["settings.textureFiltering"].status).toBe(ELevelViewerSnapshotStatus.SAME);
  });

  it("places the camera only on the level it was taken on, its lens anywhere", () => {
    const entries: Record<string, ILevelViewerSnapshotEntry> = compare(
      { camera: { fieldOfView: 60, x: 5 }, level: "levels/zaton" },
      "levels/jupiter"
    );

    expect(entries["camera.x"]).toMatchObject({
      reason: "taken on levels/zaton",
      status: ELevelViewerSnapshotStatus.SKIP,
    });
    expect(entries["camera.fieldOfView"].status).toBe(ELevelViewerSnapshotStatus.CHANGE);
  });
});
