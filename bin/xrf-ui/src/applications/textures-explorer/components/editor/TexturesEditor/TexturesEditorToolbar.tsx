import { default as CenterFocusStrongIcon } from "@mui/icons-material/CenterFocusStrong";
import { default as GrainIcon } from "@mui/icons-material/Grain";
import { default as LightbulbIcon } from "@mui/icons-material/Lightbulb";
import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { default as ViewQuiltIcon } from "@mui/icons-material/ViewQuilt";
import { Box, Divider, IconButton, Popover, ToggleButton, ToggleButtonGroup, Tooltip, Typography } from "@mui/material";
import { MouseEvent, ReactElement, useCallback, useState } from "react";

import {
  ETexturePreviewMode,
  ITexturePreviewOptions,
  TEXTURE_TILING_STEPS,
} from "@/applications/textures-explorer/lib/texture-preview";
import { ETextureSurfaceShape } from "@/applications/textures-explorer/lib/texture-surface";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { Nullable } from "@/lib/types/general";

const SHAPE_LABELS: Record<ETextureSurfaceShape, string> = {
  [ETextureSurfaceShape.PLANE]: "Plane",
  [ETextureSurfaceShape.SPHERE]: "Sphere",
  [ETextureSurfaceShape.CUBE]: "Cube",
};

/**
 * Why shading with the bump pair is unavailable, in the order a person would ask.
 *
 * @param isSurface - Whether the lit body is on screen at all.
 * @param hasBump - Whether the open texture declares a pair.
 * @param isLit - Whether a light is shading the body.
 * @returns The reason, or undefined when the toggle is available.
 */
function toBumpUnavailableTitle(isSurface: boolean, hasBump: boolean, isLit: boolean): string | undefined {
  if (!isSurface) {
    return "Lit surface only";
  }

  if (!hasBump) {
    return "This texture declares no bump pair";
  }

  return isLit ? undefined : "Nothing to shade with the light off";
}

interface ITexturesEditorToolbarProps {
  /** The open texture, as the last breadcrumb segment. */
  subtitle?: string;
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
export function TexturesEditorToolbar({
  subtitle,
  options,
  hasBump,
  onChangeOptions,
  onResetCamera,
  onBack,
}: ITexturesEditorToolbarProps): ReactElement {
  const [bodyAnchor, setBodyAnchor] = useState<Nullable<HTMLElement>>(null);

  const isSurface: boolean = options.mode === ETexturePreviewMode.SURFACE;

  const onOpenBody = useCallback((event: MouseEvent<HTMLButtonElement>) => setBodyAnchor(event.currentTarget), []);
  const onCloseBody = useCallback(() => setBodyAnchor(null), []);

  const onToggleMode = useCallback(() => {
    onChangeOptions({
      ...options,
      mode: isSurface ? ETexturePreviewMode.IMAGE : ETexturePreviewMode.SURFACE,
    });
  }, [isSurface, options, onChangeOptions]);

  return (
    <EditorToolbar
      subtitle={subtitle}
      onBack={onBack}
      actions={
        <>
          <EditorViewToggle label={"Lit surface"} icon={<ViewInArIcon />} isOn={isSurface} onToggle={onToggleMode} />

          <Divider orientation={"vertical"} flexItem sx={{ marginX: 0.5, marginY: 1 }} />

          <Tooltip title={isSurface ? `Body: ${SHAPE_LABELS[options.shape]}, ${options.tiling}×` : "Lit surface only"}>
            <span>
              <IconButton
                aria-label={"Body"}
                aria-haspopup={"dialog"}
                color={"inherit"}
                disabled={!isSurface}
                onClick={onOpenBody}
              >
                <ViewQuiltIcon />
              </IconButton>
            </span>
          </Tooltip>

          <Popover
            anchorEl={bodyAnchor}
            open={Boolean(bodyAnchor)}
            anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
            transformOrigin={{ vertical: "top", horizontal: "center" }}
            onClose={onCloseBody}
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
                onChange={(_, next: Nullable<ETextureSurfaceShape>) =>
                  next && onChangeOptions({ ...options, shape: next })
                }
              >
                {Object.values(ETextureSurfaceShape).map((value: ETextureSurfaceShape) => (
                  <ToggleButton key={value} value={value}>
                    {SHAPE_LABELS[value]}
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
          </Popover>

          <EditorViewToggle
            label={"Light"}
            icon={<LightbulbIcon />}
            isOn={options.isLit}
            isDisabled={!isSurface}
            unavailableTitle={"Lit surface only"}
            onToggle={() => onChangeOptions({ ...options, isLit: !options.isLit })}
          />

          <EditorViewToggle
            label={"Bump"}
            icon={<GrainIcon />}
            isOn={options.isBumped && options.isLit}
            isDisabled={!isSurface || !hasBump || !options.isLit}
            unavailableTitle={toBumpUnavailableTitle(isSurface, hasBump, options.isLit)}
            onToggle={() => onChangeOptions({ ...options, isBumped: !options.isBumped })}
          />

          <Tooltip title={isSurface ? "Reset camera and light" : "Lit surface only"}>
            <span>
              <IconButton aria-label={"Reset camera"} color={"inherit"} disabled={!isSurface} onClick={onResetCamera}>
                <CenterFocusStrongIcon />
              </IconButton>
            </span>
          </Tooltip>
        </>
      }
    />
  );
}
