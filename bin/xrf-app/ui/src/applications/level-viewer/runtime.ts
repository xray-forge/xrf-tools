import { LevelViewerCaptureService } from "@/applications/level-viewer/services/level-viewer-capture";
import { LevelViewerCaptureResultService } from "@/applications/level-viewer/services/level-viewer-capture-result";
import { LevelViewerListService } from "@/applications/level-viewer/services/level-viewer-list";
import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/level-viewer-screenshot";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/level-viewer-snapshot";
import { LevelViewerSpawnService } from "@/applications/level-viewer/services/level-viewer-spawn";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelLookService } from "@/core/level/services/level-look";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { LevelViewerApplication as Component } from "./LevelViewerApplication";

export const container: ContainerDefinition = {
  bindings: [
    LevelViewerListService,
    LevelLoadService,
    LevelLookService,
    LevelRenderService,
    LevelViewService,
    LevelViewerCaptureService,
    LevelViewerCaptureResultService,
    LevelViewerScreenshotService,
    LevelViewerSnapshotService,
    LevelViewerSpawnService,
    LevelViewportService,
    LevelWeatherService,
  ],
};
