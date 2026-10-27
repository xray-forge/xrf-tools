import { ContainerDefinition } from "@/lib/container/container-definition";
import { TranslationsBuilderService } from "@/tools/translations-builder/services/translations-builder.service";

export { TranslationsBuilderTool as Component } from "./TranslationsBuilderTool";

export const container: ContainerDefinition = { bindings: [TranslationsBuilderService] };
