import { useInjection } from "@wirestate/react";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { TextureSelectionService } from "@/core/textures/services/texture-selection";

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
