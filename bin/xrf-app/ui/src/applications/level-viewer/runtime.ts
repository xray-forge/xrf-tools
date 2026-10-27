import { LevelViewerCaptureService } from "@/applications/level-viewer/services/level-viewer-capture";
import { LevelViewerCaptureResultService } from "@/applications/level-viewer/services/level-viewer-capture-result";
import { LevelViewerListService } from "@/applications/level-viewer/services/level-viewer-list";
import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/level-viewer-screenshot";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/level-viewer-snapshot";
import { LevelViewerSpawnService } from "@/applications/level-viewer/services/level-viewer-spawn";
import { LevelLoadService } from "@/core/level/services/level-load/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look/level-look.service";
import { LevelRenderService } from "@/core/level/services/level-render/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather/level-weather.service";
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
