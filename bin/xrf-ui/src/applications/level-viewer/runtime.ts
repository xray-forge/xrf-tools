import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/snapshot";
import { LevelListService } from "@/core/level/services/level-list.service";
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
    LevelListService,
    LevelLoadService,
    LevelLookService,
    LevelRenderService,
    LevelViewService,
    LevelViewerSnapshotService,
    LevelViewportService,
    LevelWeatherService,
  ],
};
