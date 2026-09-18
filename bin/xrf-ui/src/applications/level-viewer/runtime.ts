import { LevelListService, LevelLoadService } from "@/core/level/services";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { LevelViewerApplication as Component } from "./LevelViewerApplication";

export const container: ContainerDefinition = {
  bindings: [LevelListService, LevelLoadService],
};
