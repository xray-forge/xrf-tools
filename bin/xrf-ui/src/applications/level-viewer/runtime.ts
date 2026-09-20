import { LevelListService, LevelLoadService, LevelViewportService } from "@/core/level/services";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { LevelViewerApplication as Component } from "./LevelViewerApplication";

export const container: ContainerDefinition = {
  bindings: [LevelListService, LevelLoadService, LevelViewportService],
};
