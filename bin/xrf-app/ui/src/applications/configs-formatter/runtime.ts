import { FormatterService } from "@/applications/configs-formatter/services/formatter";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ConfigsFormatterApplication as Component } from "./ConfigsFormatterApplication";

export const container: ContainerDefinition = { bindings: [FormatterService] };
