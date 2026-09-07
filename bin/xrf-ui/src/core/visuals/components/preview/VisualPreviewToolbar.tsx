import { default as AccountTreeIcon } from "@mui/icons-material/AccountTree";
import { default as CenterFocusStrongIcon } from "@mui/icons-material/CenterFocusStrong";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as GrainIcon } from "@mui/icons-material/Grain";
import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { default as HexagonIcon } from "@mui/icons-material/Hexagon";
import { default as OpacityIcon } from "@mui/icons-material/Opacity";
import { default as PolylineIcon } from "@mui/icons-material/Polyline";
import { default as TextureIcon } from "@mui/icons-material/Texture";
import { default as ThreeDRotationIcon } from "@mui/icons-material/ThreeDRotation";
import { Divider } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { IVisualPreviewViewOptions } from "@/core/visuals/components/scene";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { VisualMeshDetail } from "./VisualMeshDetail";

interface IVisualPreviewToolbarProps extends BaseComponentProps {
  subtitle?: string;
  options: IVisualPreviewViewOptions;
  isOpenEnabled: boolean;
  /** How far down each submesh's collapse chain the viewport is drawing: 0 is full detail, 1 is coarsest. */
  detail: number;
  /** Whether the open model carries a bind pose to draw. */
  hasSkeleton: boolean;
  /** Whether any of the open model's materials bound a bump pair to shade with. */
  hasBump: boolean;
  /** Whether any of the open model's surfaces reads its texture's alpha channel. */
  hasAlpha: boolean;
  /** Whether the open model has anything to decimate. */
  hasDetailLevels: boolean;
  onChangeOptions: (options: IVisualPreviewViewOptions) => void;
  onChangeDetail: (detail: number) => void;
  onResetCamera: () => void;
  onOpen?: () => void;
  onBrowse?: () => void;
}

/**
 * Composes scene commands, mesh detail, and view toggles in the editor toolbar.
 */
export function VisualPreviewToolbar({
  "data-testid": dataTestId,
  id,
  className,
  subtitle,
  options,
  isOpenEnabled,
  detail,
  hasDetailLevels,
  hasSkeleton,
  hasBump,
  hasAlpha,
  onChangeOptions,
  onChangeDetail,
  onResetCamera,
  onOpen,
  onBrowse,
}: IVisualPreviewToolbarProps): ReactElement {
  /**
   * Flips one view option, which is the only thing any of these toggles does.
   *
   * Keyed rather than one callback per option, because every option in this set is a boolean the toolbar owns: a
   * callback each restated the same line five times and was five places to forget a new one.
   */
  const onToggle = useCallback(
    (option: keyof IVisualPreviewViewOptions) => {
      onChangeOptions({ ...options, [option]: !options[option] });
    },
    [options, onChangeOptions]
  );

  return (
    <EditorToolbar
      data-testid={dataTestId}
      id={id}
      className={className}
      subtitle={subtitle}
      actions={
        <>
          <EditorIconAction
            label={"Open visual"}
            description={isOpenEnabled ? "Open visual" : "Open visual (not available here)"}
            icon={<FolderOpenIcon />}
            isDisabled={!isOpenEnabled}
            onClick={() => onOpen?.()}
          />

          {onBrowse ? (
            <EditorIconAction
              label={"Browse folder"}
              description={"Browse the folder this model sits in"}
              icon={<AccountTreeIcon />}
              onClick={onBrowse}
            />
          ) : null}

          <VisualMeshDetail detail={detail} hasDetailLevels={hasDetailLevels} onChange={onChangeDetail} />

          <Divider orientation={"vertical"} flexItem sx={{ marginX: 0.5, marginY: 1 }} />

          <EditorViewToggle
            label={"Wireframe"}
            icon={<HexagonIcon />}
            isOn={options.isWireframe}
            onToggle={() => onToggle("isWireframe")}
          />

          <EditorViewToggle
            label={"Uv checkerboard"}
            icon={<TextureIcon />}
            isOn={options.isCheckerVisible}
            onToggle={() => onToggle("isCheckerVisible")}
          />

          <EditorViewToggle
            label={"Alpha"}
            icon={<OpacityIcon />}
            isOn={options.isAlphaVisible}
            isDisabled={!hasAlpha}
            unavailableTitle={"No surface of this model reads alpha"}
            onToggle={() => onToggle("isAlphaVisible")}
          />

          <EditorViewToggle
            label={"Bump"}
            icon={<GrainIcon />}
            isOn={options.isBumpVisible}
            isDisabled={!hasBump}
            unavailableTitle={"No bump material in this model"}
            onToggle={() => onToggle("isBumpVisible")}
          />

          <EditorViewToggle
            label={"Skeleton"}
            icon={<PolylineIcon />}
            isOn={options.isSkeletonVisible}
            isDisabled={!hasSkeleton}
            unavailableTitle={"No skeleton in this model"}
            onToggle={() => onToggle("isSkeletonVisible")}
          />

          <EditorViewToggle
            label={"Grid"}
            icon={<GridOnIcon />}
            isOn={options.isGridVisible}
            onToggle={() => onToggle("isGridVisible")}
          />

          <EditorViewToggle
            label={"Axes"}
            icon={<ThreeDRotationIcon />}
            isOn={options.isAxesVisible}
            onToggle={() => onToggle("isAxesVisible")}
          />

          <EditorIconAction
            label={"Reset camera"}
            description={"Reset camera"}
            icon={<CenterFocusStrongIcon />}
            onClick={onResetCamera}
          />
        </>
      }
    />
  );
}
