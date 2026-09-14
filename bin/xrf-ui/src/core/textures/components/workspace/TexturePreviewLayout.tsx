import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, ReactNode, useCallback, useEffect, useState } from "react";

import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { IEditorLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { IEditorPanel, useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { TexturePreview } from "@/core/textures/components/preview/TexturePreview";
import { describeTextureCaption, describeTextureName } from "@/core/textures/lib/texture-caption";
import {
  DEFAULT_TEXTURE_PREVIEW_OPTIONS,
  ITexturePreviewComparison,
  ITexturePreviewOptions,
} from "@/core/textures/lib/texture-preview";
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
  /** Another encoding of the open texture to show beside it, for an application that can produce one. */
  comparison?: Nullable<ITexturePreviewComparison>;
  /**
   * Where the session is, for the toolbar to fall back to while no one texture is open.
   */
  sessionLocation?: Nullable<IEditorLocation>;
  /** Ends the selection without ending the session, which is what puts the file header above the viewport. */
  onDeselect?: Nullable<() => void>;
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
  comparison = null,
  sessionLocation = null,
  onDeselect = null,
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
          location={sessionLocation}
          options={previewOptions}
          hasBump={Boolean(description?.material?.bump)}
          onChangeOptions={setPreviewOptions}
          onResetCamera={onResetCamera}
          onBack={onBack}
        />
      }
      banner={banner}
    >
      <Box sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}>
        {onDeselect && description ? (
          <EditorFileHeader
            data-testid={"texture-file-header"}
            name={describeTextureName(description)}
            caption={describeTextureCaption(description.base)}
            closeLabel={"Close texture"}
            closeDescription={"Clear the selection and close this texture"}
            onClose={onDeselect}
          />
        ) : null}

        <Box sx={{ display: "flex", flexGrow: 1, minWidth: 0, minHeight: 0, overflow: "hidden" }}>
          <TexturePreview options={previewOptions} resetToken={cameraResetToken} comparison={comparison} />
        </Box>
      </Box>
    </EditorLayout>
  );
}
