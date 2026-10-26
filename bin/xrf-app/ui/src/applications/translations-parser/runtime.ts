import { TranslationsParserService } from "@/applications/translations-parser/services/parser";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { TranslationsParserApplication as Component } from "./TranslationsParserApplication";

export const container: ContainerDefinition = { bindings: [TranslationsParserService] };
