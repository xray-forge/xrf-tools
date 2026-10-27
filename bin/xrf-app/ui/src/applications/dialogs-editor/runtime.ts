import { DialogViewService } from "@/applications/dialogs-editor/services/dialog-view";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { DialogsEditorApplication as Component } from "./DialogsEditorApplication";

// The project owns which roots are open and what they hold, the view which of its dialogs is shown and how.
export const container: ContainerDefinition = { bindings: [DialogsService, DialogViewService] };
