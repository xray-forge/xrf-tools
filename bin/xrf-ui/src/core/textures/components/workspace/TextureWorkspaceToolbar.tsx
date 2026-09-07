import { default as CenterFocusStrongIcon } from "@mui/icons-material/CenterFocusStrong";
import { default as GrainIcon } from "@mui/icons-material/Grain";
import { default as LightbulbIcon } from "@mui/icons-material/Lightbulb";
import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { Divider } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarLocation, IEditorLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { ETexturePreviewMode, ITexturePreviewOptions } from "@/core/textures/lib/texture-preview";
import { Nullable } from "@/lib/types/general";

import { TextureBodyOptions } from "./TextureBodyOptions";
import { describeUnavailableBump, SURFACE_ONLY } from "./TextureWorkspaceToolbar.utils";

interface ITextureWorkspaceToolbarProps {
  /** Where the open texture is, as the last breadcrumb segment. */
  location: Nullable<IEditorLocation>;
  options: ITexturePreviewOptions;
  /** Whether the open texture declares a bump pair, which is what makes shading with one worth offering. */
  hasBump: boolean;
  onChangeOptions: (options: ITexturePreviewOptions) => void;
  onResetCamera: () => void;
  /** Closes the open texture and returns to the tree. */
  onBack: () => void;
}

/**
 * The textures explorer's toolbar: what the preview is showing, and how.
 */
export function TextureWorkspaceToolbar({
  location,
  options,
  hasBump,
  onChangeOptions,
  onResetCamera,
  onBack,
}: ITextureWorkspaceToolbarProps): ReactElement {
  const isSurface: boolean = options.mode === ETexturePreviewMode.SURFACE;

  const onToggleMode = useCallback(() => {
    onChangeOptions({
      ...options,
      mode: isSurface ? ETexturePreviewMode.IMAGE : ETexturePreviewMode.SURFACE,
    });
  }, [isSurface, options, onChangeOptions]);

  return (
    <EditorToolbar
      subtitle={location ? <EditorToolbarLocation location={location} /> : undefined}
      onBack={onBack}
      actions={
        <>
          <EditorViewToggle label={"Lit surface"} icon={<ViewInArIcon />} isOn={isSurface} onToggle={onToggleMode} />

          <Divider orientation={"vertical"} flexItem sx={{ marginX: 0.5, marginY: 1 }} />

          <TextureBodyOptions options={options} onChangeOptions={onChangeOptions} />

          <EditorViewToggle
            label={"Light"}
            icon={<LightbulbIcon />}
            isOn={options.isLit}
            isDisabled={!isSurface}
            unavailableTitle={SURFACE_ONLY}
            onToggle={() => onChangeOptions({ ...options, isLit: !options.isLit })}
          />

          <EditorViewToggle
            label={"Bump"}
            icon={<GrainIcon />}
            isOn={options.isBumped && options.isLit}
            isDisabled={!isSurface || !hasBump || !options.isLit}
            unavailableTitle={describeUnavailableBump(isSurface, hasBump, options.isLit)}
            onToggle={() => onChangeOptions({ ...options, isBumped: !options.isBumped })}
          />

          <EditorIconAction
            label={"Reset camera"}
            description={isSurface ? "Reset camera and light" : SURFACE_ONLY}
            icon={<CenterFocusStrongIcon />}
            isDisabled={!isSurface}
            onClick={onResetCamera}
          />
        </>
      }
    />
  );
}
