import { TranslationsBuilderService } from "@/applications/translations-builder/services/builder";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { TranslationsBuilderApplication as Component } from "./TranslationsBuilderApplication";

export const container: ContainerDefinition = { bindings: [TranslationsBuilderService] };
