import { beforeEach, describe, expect, it } from "@jest/globals";

import {
  ELevelViewerSnapshotStatus,
  ILevelViewerSnapshot,
  ILevelViewerSnapshotEntry,
  LEVEL_VIEWER_SNAPSHOT_FORMAT,
} from "@/applications/level-viewer/lib/level-viewer-snapshot";
import { ILevelViewerSnapshotApplied, LevelViewerSnapshotService } from "@/applications/level-viewer/services/snapshot";
import { ERenderAntialiasing } from "@/core/ipc/types/xrf-renderer";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { clearLocalStorage } from "@/core/storage";
import { ILevelViewContainerDescriptor, mockLevelViewContainer } from "@/fixtures/utils/level-container";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelViewerSnapshotService", () => {
  it("describes every setting as the view draws it, its toggles and its camera's lens", () => {
    const { container, view, renderSettings }: ILevelViewContainerDescriptor = mockLevelViewContainer([
      LevelViewerSnapshotService,
    ]);
    const snapshot: LevelViewerSnapshotService = container.get(LevelViewerSnapshotService);

    view.setFeatures({ ...view.features, reflections: { steps: 64 } });

    const described: ILevelViewerSnapshot = snapshot.describe(null);

    expect(described).toMatchObject({
      camera: { fieldOfView: view.cameraOptions.fieldOfView },
      format: LEVEL_VIEWER_SNAPSHOT_FORMAT,
      level: null,
      options: { isGrassy: view.options.isGrassy },
      settings: {
        reflections: { steps: 64 },
        shadows: { cascades: renderSettings.rendererFeatures.shadows.cascades },
      },
    });
    expect(described.camera?.x).toBeUndefined();
  });

  it("finds nothing to change in the view it was taken of", () => {
    const { container }: ILevelViewContainerDescriptor = mockLevelViewContainer([LevelViewerSnapshotService]);
    const snapshot: LevelViewerSnapshotService = container.get(LevelViewerSnapshotService);
    const entries: Array<ILevelViewerSnapshotEntry> = snapshot.compare(snapshot.describe(null));

    expect(entries.every((it: ILevelViewerSnapshotEntry) => it.status === ELevelViewerSnapshotStatus.SAME)).toBe(true);
  });

  it("imports what the view can take into the view alone, and names what it cannot", async () => {
    const { container, view, renderSettings }: ILevelViewContainerDescriptor = mockLevelViewContainer([
      LevelViewerSnapshotService,
    ]);
    const snapshot: LevelViewerSnapshotService = container.get(LevelViewerSnapshotService);
    const settings: unknown = renderSettings.rendererChoice;
    const described: ILevelViewerSnapshot = snapshot.describe(null);
    const features: IRenderFeatureSettings = renderSettings.rendererFeatures;
    const file: ILevelViewerSnapshot = {
      ...described,
      camera: { ...described.camera, fieldOfView: 60, x: 5 },
      options: { ...described.options, isGrassy: !view.options.isGrassy },
      settings: {
        ...described.settings,
        antialiasing: ERenderAntialiasing.NONE,
        reflections: { ...features.reflections, quality: "medium", steps: 64 },
        shadows: { ...features.shadows, contact: { ...features.shadows.contact, steps: 24 } },
      },
    };

    const result: ILevelViewerSnapshotApplied & { entries: Array<ILevelViewerSnapshotEntry> } =
      await snapshot.importSnapshot(file);

    expect(view.features.reflections.steps).toBe(64);
    expect(view.features.shadows.contact?.steps).toBe(24);
    expect(view.options.isGrassy).toBe(file.options?.isGrassy);
    expect(view.cameraOptions.fieldOfView).toBe(60);
    expect(view.features.antialiasing).toBeNull();
    expect(renderSettings.rendererChoice).toBe(settings);
    expect(result.applied.sort()).toEqual(
      ["camera.fieldOfView", "options.isGrassy", "settings.reflections.steps", "settings.shadows.contact.steps"].sort()
    );

    const statuses: Record<string, ELevelViewerSnapshotStatus> = Object.fromEntries(
      result.entries.map((it: ILevelViewerSnapshotEntry) => [it.key, it.status])
    );

    expect(statuses["camera.x"]).toBe(ELevelViewerSnapshotStatus.SKIP);
    expect(statuses["settings.antialiasing"]).toBe(ELevelViewerSnapshotStatus.SKIP);
    expect(statuses["settings.reflections.quality"]).toBe(ELevelViewerSnapshotStatus.SKIP);
  });
});
