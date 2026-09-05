import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { AssetService } from "@/core/assets/services";
import { VisualLoadService } from "@/core/visuals/services";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ArchivesExplorerApplication as Component } from "./ArchivesExplorerApplication";

export const container: ContainerDefinition = {
  bindings: [AssetService, ArchivesService, VisualLoadService],
};
