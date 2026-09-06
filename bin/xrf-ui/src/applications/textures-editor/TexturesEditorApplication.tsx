import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { TexturesEditorOpenForm } from "@/applications/textures-editor/components/TexturesEditorOpenForm";
import { TexturesEditorWorkspace } from "@/applications/textures-editor/components/TexturesEditorWorkspace";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ITexturesEditorApplicationProps extends BaseComponentProps {}

/**
 * A workbench for one texture: what it is, what its descriptor declares, and what an operation would make of it.
 */
export function TexturesEditorApplication({
  "data-testid": dataTestId = "textures-editor-application",
}: ITexturesEditorApplicationProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);

  if (selectionService.selected.value === null) {
    return <TexturesEditorOpenForm data-testid={dataTestId} />;
  }

  return <TexturesEditorWorkspace data-testid={dataTestId} />;
}
