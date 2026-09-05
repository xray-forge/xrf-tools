import { TranslationsFormatterService } from "@/applications/translations-formatter/services/formatter";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { TranslationsFormatterApplication as Component } from "./TranslationsFormatterApplication";

export const container: ContainerDefinition = { bindings: [TranslationsFormatterService] };
