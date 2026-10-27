import { ContainerDefinition } from "@/lib/container/container-definition";
import { ConfigsFormatterService } from "@/tools/configs-formatter/services/configs-formatter.service";

export { ConfigsFormatterTool as Component } from "./ConfigsFormatterTool";

export const container: ContainerDefinition = { bindings: [ConfigsFormatterService] };
