import { default as AccountTreeIcon } from "@mui/icons-material/AccountTree";
import { default as CenterFocusStrongIcon } from "@mui/icons-material/CenterFocusStrong";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { default as HexagonIcon } from "@mui/icons-material/Hexagon";
import { default as TextureIcon } from "@mui/icons-material/Texture";
import { default as ThreeDRotationIcon } from "@mui/icons-material/ThreeDRotation";
import { default as TuneIcon } from "@mui/icons-material/Tune";
import { Box, Divider, IconButton, Popover, Slider, Tooltip, Typography } from "@mui/material";
import { MouseEvent, ReactElement, useCallback, useState } from "react";

import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { LAYOUT } from "@/core/theme/tokens";
import { IVisualPreviewViewOptions } from "@/core/visuals/components/scene";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IVisualPreviewToolbarProps extends BaseComponentProps {
  subtitle?: string;
  options: IVisualPreviewViewOptions;
  isOpenEnabled: boolean;
  /** How far down each submesh's collapse chain the viewport is drawing: 0 is full detail, 1 is coarsest. */
  detail: number;
  /** Whether the open model has anything to decimate. */
  hasDetailLevels: boolean;
  onChangeOptions: (options: IVisualPreviewViewOptions) => void;
  onChangeDetail: (detail: number) => void;
  onResetCamera: () => void;
  onOpen?: () => void;
  onBrowse?: () => void;
}

/**
 * View toggles and the detail control, both live and driving the scene.
 *
 * Detail is a slider rather than a list of levels because an X-Ray slide-window table is one entry per edge collapse:
 * a measured character submesh carries 948 of them, so there is nothing to enumerate. Moving it costs a draw range
 * and nothing else, since every level is already in the uploaded index buffer. A model with nothing to decimate shows
 * the control disabled rather than hidden, so the toolbar does not change shape as the user steps through a tree.
 */
export function VisualPreviewToolbar({
  subtitle,
  options,
  isOpenEnabled,
  detail,
  hasDetailLevels,
  onChangeOptions,
  onChangeDetail,
  onResetCamera,
  onOpen,
  onBrowse,
}: IVisualPreviewToolbarProps): ReactElement {
  const [detailAnchor, setDetailAnchor] = useState<Nullable<HTMLElement>>(null);

  const onOpenDetail = useCallback((event: MouseEvent<HTMLButtonElement>) => {
    setDetailAnchor(event.currentTarget);
  }, []);

  const onCloseDetail = useCallback(() => setDetailAnchor(null), []);

  /**
   * The slider reads as quality - right is the full mesh - while the stored value is how far down each collapse chain
   * to go, so the two are inverses of each other. Inverting only one of them draws 25% quality as 25% decimation.
   */
  const onSlideDetail = useCallback(
    (_: Event, value: number | Array<number>) => {
      onChangeDetail(1 - (value as number) / 100);
    },
    [onChangeDetail]
  );

  const onToggleWireframe = useCallback(() => {
    onChangeOptions({ ...options, isWireframe: !options.isWireframe });
  }, [options, onChangeOptions]);

  const onToggleGrid = useCallback(() => {
    onChangeOptions({ ...options, isGridVisible: !options.isGridVisible });
  }, [options, onChangeOptions]);

  const onToggleAxes = useCallback(() => {
    onChangeOptions({ ...options, isAxesVisible: !options.isAxesVisible });
  }, [options, onChangeOptions]);

  const onToggleChecker = useCallback(() => {
    onChangeOptions({ ...options, isCheckerVisible: !options.isCheckerVisible });
  }, [options, onChangeOptions]);

  return (
    <EditorToolbar
      subtitle={subtitle}
      actions={
        <>
          <Tooltip title={isOpenEnabled ? "Open visual" : "Open visual (not available here)"}>
            <span>
              <IconButton aria-label={"Open visual"} color={"inherit"} disabled={!isOpenEnabled} onClick={onOpen}>
                <FolderOpenIcon />
              </IconButton>
            </span>
          </Tooltip>

          {onBrowse ? (
            <Tooltip title={"Browse the folder this model sits in"}>
              <IconButton aria-label={"Browse folder"} color={"inherit"} onClick={onBrowse}>
                <AccountTreeIcon />
              </IconButton>
            </Tooltip>
          ) : null}

          <Tooltip
            title={hasDetailLevels ? `Mesh detail: ${Math.round((1 - detail) * 100)}%` : "Nothing to decimate"}
            describeChild
          >
            <span>
              <IconButton
                aria-label={"Mesh detail"}
                aria-haspopup={"dialog"}
                color={detail && hasDetailLevels ? "primary" : "inherit"}
                disabled={!hasDetailLevels}
                onClick={onOpenDetail}
              >
                <TuneIcon />
              </IconButton>
            </span>
          </Tooltip>

          <Popover
            anchorEl={detailAnchor}
            open={Boolean(detailAnchor)}
            anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
            transformOrigin={{ vertical: "top", horizontal: "center" }}
            onClose={onCloseDetail}
          >
            <Box sx={{ paddingX: 2, paddingY: 1, width: LAYOUT.toolbarSliderWidth }}>
              <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
                Mesh detail
              </Typography>

              <Slider
                size={"small"}
                min={0}
                max={100}
                value={Math.round((1 - detail) * 100)}
                valueLabelDisplay={"auto"}
                valueLabelFormat={(value: number) => `${value}%`}
                aria-label={"Mesh detail"}
                onChange={onSlideDetail}
              />
            </Box>
          </Popover>

          <Divider orientation={"vertical"} flexItem sx={{ marginX: 0.5, marginY: 1 }} />

          <Tooltip title={"Wireframe"}>
            <IconButton
              aria-label={"Wireframe"}
              color={"inherit"}
              sx={{ opacity: options.isWireframe ? 1 : 0.45 }}
              onClick={onToggleWireframe}
            >
              <HexagonIcon />
            </IconButton>
          </Tooltip>

          <Tooltip title={"Uv checkerboard"}>
            <IconButton
              aria-label={"Uv checkerboard"}
              color={"inherit"}
              sx={{ opacity: options.isCheckerVisible ? 1 : 0.45 }}
              onClick={onToggleChecker}
            >
              <TextureIcon />
            </IconButton>
          </Tooltip>

          <Tooltip title={"Grid"}>
            <IconButton
              aria-label={"Grid"}
              sx={{ opacity: options.isGridVisible ? 1 : 0.45 }}
              color={"inherit"}
              onClick={onToggleGrid}
            >
              <GridOnIcon />
            </IconButton>
          </Tooltip>

          <Tooltip title={"Axes"}>
            <IconButton
              aria-label={"Axes"}
              sx={{ opacity: options.isAxesVisible ? 1 : 0.45 }}
              color={"inherit"}
              onClick={onToggleAxes}
            >
              <ThreeDRotationIcon />
            </IconButton>
          </Tooltip>

          <Tooltip title={"Reset camera"}>
            <IconButton aria-label={"Reset camera"} color={"inherit"} onClick={onResetCamera}>
              <CenterFocusStrongIcon />
            </IconButton>
          </Tooltip>
        </>
      }
    />
  );
}
