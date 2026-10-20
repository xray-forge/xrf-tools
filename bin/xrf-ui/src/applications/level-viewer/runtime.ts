import { LevelViewerListService } from "@/applications/level-viewer/services/list";
import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/screenshot";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/snapshot";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { LevelViewerApplication as Component } from "./LevelViewerApplication";

export const container: ContainerDefinition = {
  bindings: [
    LevelViewerListService,
    LevelLoadService,
    LevelLookService,
    LevelRenderService,
    LevelViewService,
    LevelViewerScreenshotService,
    LevelViewerSnapshotService,
    LevelViewportService,
    LevelWeatherService,
  ],
};
