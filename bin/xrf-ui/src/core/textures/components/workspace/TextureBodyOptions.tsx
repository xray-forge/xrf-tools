import { default as ViewQuiltIcon } from "@mui/icons-material/ViewQuilt";
import { Box, ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { ReactElement } from "react";

import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { ETexturePreviewMode, ITexturePreviewOptions, TEXTURE_TILING_STEPS } from "@/core/textures/lib/texture-preview";
import { describeTextureSurfaceShape, ETextureSurfaceShape } from "@/core/textures/lib/texture-surface";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
      description={isSurface ? `Body: ${describeTextureSurfaceShape(options.shape)}, ${options.tiling}×` : SURFACE_ONLY}
      icon={<ViewQuiltIcon />}
      isDisabled={!isSurface}
    >
      <Box sx={{ display: "flex", flexDirection: "column", gap: 1, paddingX: 2, paddingY: 1 }}>
        <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
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

        <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
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
      </Box>
    </EditorPopoverAction>
  );
}
