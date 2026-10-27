import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelLookService } from "@/core/level/services/level-look";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { ContainerDefinition } from "@/lib/container/container-definition";
import { LevelViewerCaptureService } from "@/tools/level-viewer/services/level-viewer-capture";
import { LevelViewerCaptureResultService } from "@/tools/level-viewer/services/level-viewer-capture-result";
import { LevelViewerListService } from "@/tools/level-viewer/services/level-viewer-list";
import { LevelViewerScreenshotService } from "@/tools/level-viewer/services/level-viewer-screenshot";
import { LevelViewerSnapshotService } from "@/tools/level-viewer/services/level-viewer-snapshot";
import { LevelViewerSpawnService } from "@/tools/level-viewer/services/level-viewer-spawn";

export { LevelViewerTool as Component } from "./LevelViewerTool";

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
