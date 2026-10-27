import { ConfigsFormatterService } from "@/applications/configs-formatter/services/configs-formatter.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ConfigsFormatterApplication as Component } from "./ConfigsFormatterApplication";

export const container: ContainerDefinition = { bindings: [ConfigsFormatterService] };
