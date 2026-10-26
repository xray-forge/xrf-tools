import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { TextureSelectionService } from "@/core/textures/services/selection";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { TexturesEditorOpenForm } from "./components/TexturesEditorOpenForm";
import { TexturesEditorWorkspace } from "./components/TexturesEditorWorkspace";

export function TexturesEditorApplication({
  "data-testid": dataTestId = "textures-editor-application",
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);

  if (selectionService.selected.value === null) {
    return <TexturesEditorOpenForm data-testid={dataTestId} />;
  }

  return <TexturesEditorWorkspace data-testid={dataTestId} />;
}
