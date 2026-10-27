import { useInjection } from "@wirestate/react";
import { TextureDescription } from "@xrf/ipc/types/xrf-app-textures";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { TEXTURES_EDITOR_PANELS } from "@/applications/textures-editor/components/panels/textures-editor-panels";
import { TextureBumpService } from "@/applications/textures-editor/services/texture-bump";
import { TextureEditorService } from "@/applications/textures-editor/services/texture-editor";
import { TextureEncodingService } from "@/applications/textures-editor/services/texture-encoding";
import { EditorSaver, useEditorLifecycle } from "@/core/shell/editor-lifecycle";
import { IEditorPanel } from "@/core/shell/editor-shell";
import { TexturePreviewLayout } from "@/core/textures/components/workspace/TexturePreviewLayout";
import { ITexturePreviewComparison } from "@/core/textures/lib/texture-preview";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";

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
  const bumpService: TextureBumpService = useInjection(TextureBumpService);

  const description: Nullable<TextureDescription> = selectionService.selected.value;

  // A generated pair is written to disk the moment its job finishes, and a comparison is a job the backend keeps
  // running, so both hold the editor open as a save does.
  const isBusy: boolean =
    editorService.save.isRunning || encodingService.compare.isRunning || bumpService.generate.isRunning;

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
    isBusy,
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
