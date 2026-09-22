import { default as GrainIcon } from "@mui/icons-material/Grain";
import { default as LightbulbIcon } from "@mui/icons-material/Lightbulb";
import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

import { RenderLightingAction } from "@/core/render/components/lighting/RenderLightingAction";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarLocation, IEditorLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { EditorToolbarSeparator } from "@/core/shell/editor/EditorToolbarSeparator";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { DEFAULT_TEXTURE_LIGHTING } from "@/core/textures/lib/scene/texture-lighting";
import { ETexturePreviewMode, ITexturePreviewOptions } from "@/core/textures/lib/texture-preview";

import { TextureBodyOptions } from "./TextureBodyOptions";
import { describeUnavailableBump, SURFACE_ONLY } from "./TextureWorkspaceToolbar.utils";

interface ITextureWorkspaceToolbarProps {
  /** Where the open texture is, as the last breadcrumb segment. */
  location: Nullable<IEditorLocation>;
  options: ITexturePreviewOptions;
  /** What the lit body is lit with, which a drag over it also changes. */
  lighting: IRenderLighting;
  /** Whether the open texture declares a bump pair, which is what makes shading with one worth offering. */
  hasBump: boolean;
  onChangeOptions: (options: ITexturePreviewOptions) => void;
  onChangeLighting: (lighting: IRenderLighting) => void;
  /** Closes the open texture and returns to the tree. */
  onBack: () => void;
}

/**
 * The textures explorer's toolbar: what the preview is showing, and how.
 */
export function TextureWorkspaceToolbar({
  location,
  options,
  lighting,
  hasBump,
  onChangeOptions,
  onChangeLighting,
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

          <EditorToolbarSeparator />

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

          <TextureBodyOptions options={options} onChangeOptions={onChangeOptions} />

          <RenderLightingAction
            lighting={lighting}
            fallback={DEFAULT_TEXTURE_LIGHTING}
            isDisabled={!isSurface}
            unavailableDescription={SURFACE_ONLY}
            onChange={onChangeLighting}
          />
        </>
      }
    />
  );
}
