import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { TEXTURES_EDITOR_PANELS } from "@/applications/textures-editor/components/panels/textures-editor-panels";
import { TextureEditorService } from "@/applications/textures-editor/services/editor";
import { TextureEncodingService } from "@/applications/textures-editor/services/encoding";
import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { EditorSaver, useEditorLifecycle } from "@/core/shell/editor-lifecycle";
import { IEditorPanel } from "@/core/shell/editor-shell";
import { TexturePreviewLayout } from "@/core/textures/components/workspace/TexturePreviewLayout";
import { ITexturePreviewComparison } from "@/core/textures/lib/texture-preview";
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
  const editorService: TextureEditorService = useInjection(TextureEditorService);
  const encodingService: TextureEncodingService = useInjection(TextureEncodingService);

  const description: Nullable<TextureDescription> = selectionService.selected.value;

  const panels: Array<IEditorPanel> = useMemo(() => TEXTURES_EDITOR_PANELS, []);

  // Only while a candidate is held. Weighing alone puts numbers in a panel; choosing one is what says somebody wants
  // to look at what it would do to the picture.
  const comparison: Nullable<ITexturePreviewComparison> = encodingService.chosenReport
    ? { label: encodingService.chosenReport.label, preview: encodingService.preview }
    : null;

  const onSave: EditorSaver = useCallback(async () => {
    await editorService.commit();

    return !editorService.isDirty;
  }, [editorService]);

  const onBack = useCallback(() => selectionService.clear(), [selectionService]);

  useEffect(() => editorService.bind(description), [editorService, description]);

  useEditorLifecycle({
    isBusy: editorService.save.isRunning,
    dirtyCount: editorService.isDirty ? 1 : 0,
    save: editorService.canSave ? onSave : null,
  });

  return (
    <TexturePreviewLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      panels={panels}
      status={describeEditedTextureStatus(description)}
      comparison={comparison}
      onBack={onBack}
    />
  );
}
