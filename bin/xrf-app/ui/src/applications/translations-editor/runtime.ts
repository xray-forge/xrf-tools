import { TranslationsService } from "@/applications/translations-editor/services/translations";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { TranslationsEditorApplication as Component } from "./TranslationsEditorApplication";

export const container: ContainerDefinition = { bindings: [TranslationsService] };
