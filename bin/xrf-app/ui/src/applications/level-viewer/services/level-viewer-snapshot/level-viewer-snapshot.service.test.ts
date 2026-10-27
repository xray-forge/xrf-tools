import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";
import { ERenderAntialiasing } from "@xrf/ipc/types/xrf-renderer";

import {
  ELevelViewerSnapshotStatus,
  ILevelViewerSnapshot,
  ILevelViewerSnapshotApplied,
  ILevelViewerSnapshotEntry,
  LEVEL_VIEWER_SNAPSHOT_FORMAT,
} from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/level-viewer-snapshot";
import { ILevelViewContainerDescriptor, mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EApplicationId } from "@/core/routing/application";
import { clearLocalStorage } from "@/core/storage";
import { mockApplicationScope } from "@/fixtures/utils/container";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelViewerSnapshotService", () => {
  it("applies the values chosen of a file once, however often asked while they are applied", async () => {
    const { container }: ILevelViewContainerDescriptor = mockLevelViewContainer([LevelViewerSnapshotService]);
    const snapshot: LevelViewerSnapshotService = container.get(LevelViewerSnapshotService);
    const applied: ILevelViewerSnapshotApplied = { applied: ["camera.fieldOfView"], failed: [] };
    const apply = jest.spyOn(snapshot, "apply").mockResolvedValue(applied);

    runInAction(() => {
      snapshot.importing = { entries: [], fileName: "zaton.json", snapshot: { format: LEVEL_VIEWER_SNAPSHOT_FORMAT } };
    });

    await Promise.all([snapshot.applyImport([]), snapshot.applyImport([])]);

    expect(apply).toHaveBeenCalledTimes(1);
    expect(snapshot.applied.value).toEqual(applied);
    expect(snapshot.applied.isLoading).toBe(false);
  });

  // The dialog showed values it could not apply; its notice says why, and the dialog closes.
  it("closes an import that failed and says why from the viewer", async () => {
    const { container }: ILevelViewContainerDescriptor = mockLevelViewContainer([
      ...mockApplicationScope(EApplicationId.LEVEL_VIEWER),
      LevelViewerSnapshotService,
    ]);
    const snapshot: LevelViewerSnapshotService = container.get(LevelViewerSnapshotService);
    const raised: Array<INotificationPayload> = [];

    container
      .get(EventBus)
      .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
        raised.push(event.payload as INotificationPayload)
      );
    jest.spyOn(snapshot.log, "error").mockImplementation(() => undefined);
    jest.spyOn(snapshot, "apply").mockRejectedValue(new Error("the weather is not read"));
    runInAction(() => {
      snapshot.importing = { entries: [], fileName: "zaton.json", snapshot: { format: LEVEL_VIEWER_SNAPSHOT_FORMAT } };
    });

    await snapshot.applyImport([]);

    expect(snapshot.importing).toBeNull();
    expect(snapshot.applied.state.isIdle).toBe(true);
    expect(raised).toEqual([
      expect.objectContaining({
        details: "zaton.json\nthe weather is not read",
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.LEVEL_VIEWER,
        title: "Could not import the view",
      }),
    ]);
  });

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
