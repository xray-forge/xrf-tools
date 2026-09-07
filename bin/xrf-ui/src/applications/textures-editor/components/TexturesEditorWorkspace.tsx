import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { createTexturesEditorPanels } from "@/applications/textures-editor/components/panels/textures-editor-panels";
import { TextureEditorService } from "@/applications/textures-editor/services/editor";
import { TextureEncodingService } from "@/applications/textures-editor/services/encoding";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { EditorSaver, useEditorLifecycle } from "@/core/shell/editor-lifecycle";
import { IEditorPanel } from "@/core/shell/panel/context";
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

  const panels: Array<IEditorPanel> = useMemo(() => createTexturesEditorPanels(), []);

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

  // The draft belongs to the texture that is open rather than to whichever panel is on screen, so it is bound here.
  // Bound from a panel it would exist only while that panel is mounted, and everything downstream of it - the save,
  // the leave prompt - would be answering about a draft that may not have been made yet.
  useEffect(() => editorService.bind(description), [editorService, description]);

  // One texture, so the count is one or none. The saver is published only when there is somewhere to write: a texture
  // served out of an archive can be edited and read, and the prompt says so by offering nothing but discarding.
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
