import { ContainerDefinition } from "@/lib/container/container-definition";
import { TranslationsService } from "@/tools/translations-editor/services/translations.service";

export { TranslationsEditorTool as Component } from "./TranslationsEditorTool";

export const container: ContainerDefinition = { bindings: [TranslationsService] };
