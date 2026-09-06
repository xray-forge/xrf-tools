import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { createTexturesEditorPanels } from "@/applications/textures-editor/components/panels/textures-editor-panels";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { IEditorPanel } from "@/core/shell/panel/context";
import { TexturePreviewLayout } from "@/core/textures/components/workspace/TexturePreviewLayout";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeEditedTextureStatus } from "./TexturesEditorWorkspace.utils";

/**
 * The workbench: one texture, what it is, and what an operation would make of it.
 */
export function TexturesEditorWorkspace({
  "data-testid": dataTestId = "textures-editor-workspace",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);

  const description: Nullable<TextureDescription> = selectionService.selected.value;

  const panels: Array<IEditorPanel> = useMemo(() => createTexturesEditorPanels(), []);
  const onBack = useCallback(() => selectionService.clear(), [selectionService]);

  return (
    <TexturePreviewLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      panels={panels}
      status={describeEditedTextureStatus(description)}
      onBack={onBack}
    />
  );
}
