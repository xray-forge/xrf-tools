import { useInjection } from "@wirestate/react";
import { ReactElement, ReactNode, useCallback, useEffect, useState } from "react";

import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { useEditorStatus } from "@/core/shell/EditorStatusContext";
import { IEditorPanel, useEditorPanels } from "@/core/shell/panel/context";
import { TexturePreview } from "@/core/textures/components/preview/TexturePreview";
import { DEFAULT_TEXTURE_PREVIEW_OPTIONS, ITexturePreviewOptions } from "@/core/textures/lib/texture-preview";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { TextureWorkspaceToolbar } from "./TextureWorkspaceToolbar";

interface ITexturePreviewLayoutProps extends BaseComponentProps {
  /** The panels this application offers, already built. */
  panels: Array<IEditorPanel>;
  /** What the status bar says about the session, which only the application knows. */
  status: Array<string>;
  /** Shown above the viewport, for a notice that belongs to the session rather than to the texture. */
  banner?: ReactNode;
  /** Leaving the session, which each application ends its own way. */
  onBack: () => void;
}

/**
 * Everything both texture applications put around a texture: the viewport, its options, and the panels beside it.
 */
export function TexturePreviewLayout({
  "data-testid": dataTestId = "texture-preview-layout",
  id,
  className,
  panels,
  status,
  banner,
  onBack,
}: ITexturePreviewLayoutProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const surfaceService: TextureSurfaceService = useInjection(TextureSurfaceService);

  const [previewOptions, setPreviewOptions] = useState<ITexturePreviewOptions>(DEFAULT_TEXTURE_PREVIEW_OPTIONS);
  const [cameraResetToken, setCameraResetToken] = useState<number>(0);

  const description: Nullable<TextureDescription> = selectionService.selected.value;

  const onResetCamera = useCallback(() => setCameraResetToken((it: number) => it + 1), []);

  // Uploaded for whichever texture is open rather than by whatever happens to be drawing it, because the lit surface
  // and the channel panel read the same pair and either of the two can be the only one on screen.
  useEffect(() => {
    if (description) {
      void surfaceService.load(description);
    } else {
      surfaceService.clear();
    }
  }, [surfaceService, description]);

  useEffect(() => () => surfaceService.clear(), [surfaceService]);

  useEditorPanels(() => panels, [panels]);

  useEditorStatus(status);

  return (
    <EditorLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      toolbar={
        <TextureWorkspaceToolbar
          subtitle={selectionService.reference ?? undefined}
          options={previewOptions}
          hasBump={Boolean(description?.material.bump)}
          onChangeOptions={setPreviewOptions}
          onResetCamera={onResetCamera}
          onBack={onBack}
        />
      }
      banner={banner}
    >
      <TexturePreview options={previewOptions} resetToken={cameraResetToken} />
    </EditorLayout>
  );
}
