import { default as CenterFocusStrongIcon } from "@mui/icons-material/CenterFocusStrong";
import { default as GrainIcon } from "@mui/icons-material/Grain";
import { default as LightbulbIcon } from "@mui/icons-material/Lightbulb";
import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { default as ViewQuiltIcon } from "@mui/icons-material/ViewQuilt";
import { Box, Divider, IconButton, Popover, ToggleButton, ToggleButtonGroup, Tooltip, Typography } from "@mui/material";
import { MouseEvent, ReactElement, useCallback, useState } from "react";

import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { ETexturePreviewMode, ITexturePreviewOptions, TEXTURE_TILING_STEPS } from "@/core/textures/lib/texture-preview";
import { describeTextureSurfaceShape, ETextureSurfaceShape } from "@/core/textures/lib/texture-surface";
import { Nullable } from "@/lib/types/general";

import { describeUnavailableBump, SURFACE_ONLY } from "./TextureWorkspaceToolbar.utils";

interface ITextureWorkspaceToolbarProps {
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
export function TextureWorkspaceToolbar({
  subtitle,
  options,
  hasBump,
  onChangeOptions,
  onResetCamera,
  onBack,
}: ITextureWorkspaceToolbarProps): ReactElement {
  const [bodyAnchor, setBodyAnchor] = useState<Nullable<HTMLElement>>(null);

  const isSurface: boolean = options.mode === ETexturePreviewMode.SURFACE;
  const bodyTitle: string = isSurface
    ? `Body: ${describeTextureSurfaceShape(options.shape)}, ${options.tiling}×`
    : SURFACE_ONLY;

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

          <Tooltip title={bodyTitle}>
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
          </Popover>

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

          <Tooltip title={isSurface ? "Reset camera and light" : SURFACE_ONLY}>
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
