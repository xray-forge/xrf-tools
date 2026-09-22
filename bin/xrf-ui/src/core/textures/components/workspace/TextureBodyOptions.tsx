import { default as ViewQuiltIcon } from "@mui/icons-material/ViewQuilt";
import { ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { ETexturePreviewMode, ITexturePreviewOptions, TEXTURE_TILING_STEPS } from "@/core/textures/lib/texture-preview";
import {
  describeTextureSurfaceAlpha,
  describeTextureSurfaceShape,
  ETextureSurfaceAlpha,
  ETextureSurfaceShape,
} from "@/core/textures/lib/texture-surface";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { SURFACE_ONLY } from "./TextureWorkspaceToolbar.utils";

interface ITextureBodyOptionsProps extends BaseComponentProps {
  options: ITexturePreviewOptions;
  onChangeOptions: (options: ITexturePreviewOptions) => void;
}

/**
 * Shape and tiling controls for the texture's lit surface preview.
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
      <div className={"flex flex-col gap-2 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Body
        </Typography>

        <ToggleButtonGroup
          exclusive
          size={"small"}
          value={options.shape}
          aria-label={"Body shape"}
          onChange={(_, next: Nullable<ETextureSurfaceShape>) => next && onChangeOptions({ ...options, shape: next })}
        >
          {Object.values(ETextureSurfaceShape).map((value: ETextureSurfaceShape) => (
            <ToggleButton key={value} value={value}>
              {describeTextureSurfaceShape(value)}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>

        <Typography className={"text-text-secondary"} variant={"overline"}>
          Tiling
        </Typography>

        <ToggleButtonGroup
          exclusive
          size={"small"}
          value={options.tiling}
          aria-label={"Tiling"}
          onChange={(_, next: Nullable<number>) => next && onChangeOptions({ ...options, tiling: next })}
        >
          {TEXTURE_TILING_STEPS.map((value: number) => (
            <ToggleButton key={value} value={value} aria-label={`Tile ${value} by ${value}`}>
              {`${value}×`}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>

        <Typography className={"text-text-secondary"} variant={"overline"}>
          Alpha
        </Typography>

        <ToggleButtonGroup
          exclusive
          size={"small"}
          value={options.alpha}
          aria-label={"Alpha"}
          onChange={(_, next: Nullable<ETextureSurfaceAlpha>) => next && onChangeOptions({ ...options, alpha: next })}
        >
          {Object.values(ETextureSurfaceAlpha).map((value: ETextureSurfaceAlpha) => (
            <ToggleButton key={value} value={value}>
              {describeTextureSurfaceAlpha(value)}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </div>
    </EditorPopoverAction>
  );
}
