import { ETextureSurfaceAlpha, ETextureSurfaceShape } from "@xrf/ipc/types/xrf-app-textures";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { EditorPopoverContent } from "@xrf/ui/editor/popover-content";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { ViewQuiltIcon } from "@xrf/ui/icons/view-quilt";
import { ReactElement } from "react";

import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { ETexturePreviewMode, ITexturePreviewOptions, TEXTURE_TILING_STEPS } from "@/core/textures/lib/texture-preview";
import { describeTextureSurfaceAlpha, describeTextureSurfaceShape } from "@/core/textures/lib/texture-surface";

import { SURFACE_ONLY } from "./TextureWorkspaceToolbar.utils";

/** Every body a texture can be laid on. */
const SHAPE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ETextureSurfaceShape>> = Object.values(
  ETextureSurfaceShape
).map((value: ETextureSurfaceShape) => ({ label: describeTextureSurfaceShape(value), value }));

/** Every way the body reads the texture's alpha. */
const ALPHA_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ETextureSurfaceAlpha>> = Object.values(
  ETextureSurfaceAlpha
).map((value: ETextureSurfaceAlpha) => ({ label: describeTextureSurfaceAlpha(value), value }));

/** Every tiling offered, as the choice's text values; a choice holds strings, the preview a count. */
const TILING_OPTIONS: ReadonlyArray<IChoiceFormRowOption<string>> = TEXTURE_TILING_STEPS.map((value: number) => ({
  "aria-label": `Tile ${value} by ${value}`,
  label: `${value}×`,
  value: String(value),
}));

interface ITextureBodyOptionsProps extends BaseComponentProps {
  options: ITexturePreviewOptions;
  onChangeOptions: (options: ITexturePreviewOptions) => void;
}

/**
 * Shape, tiling and alpha of the texture's lit surface preview, a row each.
 */
export function TextureBodyOptions({
  "data-testid": dataTestId = "texture-body-options",
  id,
  className,
  options,
  onChangeOptions,
}: ITextureBodyOptionsProps): ReactElement {
  const isSurface: boolean = options.mode === ETexturePreviewMode.SURFACE;

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Body"}
      description={
        isSurface
          ? `Body: ${describeTextureSurfaceShape(options.shape)}, ${options.tiling}×, alpha ${describeTextureSurfaceAlpha(
              options.alpha
            ).toLowerCase()}`
          : SURFACE_ONLY
      }
      icon={<ViewQuiltIcon />}
      isDisabled={!isSurface}
    >
      <EditorPopoverContent label={"Body"}>
        <RenderValueChoice
          label={"Shape"}
          options={SHAPE_OPTIONS}
          value={options.shape}
          onChange={(shape: ETextureSurfaceShape) => onChangeOptions({ ...options, shape })}
        />

        <RenderValueChoice
          label={"Tiling"}
          options={TILING_OPTIONS}
          value={String(options.tiling)}
          onChange={(tiling: string) => onChangeOptions({ ...options, tiling: Number(tiling) })}
        />

        <RenderValueChoice
          label={"Alpha"}
          options={ALPHA_OPTIONS}
          value={options.alpha}
          onChange={(alpha: ETextureSurfaceAlpha) => onChangeOptions({ ...options, alpha })}
        />
      </EditorPopoverContent>
    </EditorPopoverAction>
  );
}
