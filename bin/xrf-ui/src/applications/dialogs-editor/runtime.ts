import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { DialogsEditorApplication as Component } from "./DialogsEditorApplication";

export const container: ContainerDefinition = { bindings: [DialogsService] };
