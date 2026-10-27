import { ExportsService } from "@/applications/exports-explorer/services/exports.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ExportsExplorerApplication as Component } from "./ExportsExplorerApplication";

export const container: ContainerDefinition = { bindings: [ExportsService] };
