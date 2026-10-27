import { ContainerDefinition } from "@/lib/container/container-definition";
import { ExportsService } from "@/tools/exports-explorer/services/exports.service";

export { ExportsExplorerTool as Component } from "./ExportsExplorerTool";

export const container: ContainerDefinition = { bindings: [ExportsService] };
