import { ContainerDefinition } from "@/lib/container/container-definition";
import { DialogViewService } from "@/tools/dialogs-editor/services/dialog-view";
import { DialogsService } from "@/tools/dialogs-editor/services/dialogs";

export { DialogsEditorTool as Component } from "./DialogsEditorTool";

// The project owns which roots are open and what they hold, the view which of its dialogs is shown and how.
export const container: ContainerDefinition = { bindings: [DialogsService, DialogViewService] };
