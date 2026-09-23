import { default as FoggyIcon } from "@mui/icons-material/Foggy";
import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { default as HexagonIcon } from "@mui/icons-material/Hexagon";
import { default as LightbulbIcon } from "@mui/icons-material/Lightbulb";
import { default as QueryStatsIcon } from "@mui/icons-material/QueryStats";
import { default as TextureIcon } from "@mui/icons-material/Texture";
import { default as ThreeDRotationIcon } from "@mui/icons-material/ThreeDRotation";
import { default as WbSunnyIcon } from "@mui/icons-material/WbSunny";
import { ReactElement, ReactNode, useCallback } from "react";

import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarSeparator } from "@/core/shell/editor/EditorToolbarSeparator";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewToolbarProps extends BaseComponentProps {
  subtitle?: ReactNode;
  options: ILevelViewOptions;
  /** Value pickers the surface contributes, drawn last, as every toolbar in this application orders them. */
  actions?: ReactNode;
  onChangeOptions: (options: ILevelViewOptions) => void;
  onBack?: () => void;
}

/**
 * Composes the level view toggles in the editor toolbar.
 */
export function LevelPreviewToolbar({
  "data-testid": dataTestId,
  id,
  className,
  subtitle,
  options,
  actions,
  onChangeOptions,
  onBack,
}: ILevelPreviewToolbarProps): ReactElement {
  const onToggle = useCallback(
    (option: keyof ILevelViewOptions) => {
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
      onBack={onBack}
      actions={
        <>
          <EditorViewToggle
            label={"Wireframe"}
            icon={<HexagonIcon />}
            isOn={options.isWireframe}
            onToggle={() => onToggle("isWireframe")}
          />

          <EditorViewToggle
            label={"Textures"}
            icon={<TextureIcon />}
            isOn={options.isTextured}
            onToggle={() => onToggle("isTextured")}
          />

          <EditorToolbarSeparator />

          <EditorViewToggle
            label={"Baked light"}
            icon={<LightbulbIcon />}
            isOn={options.isLit}
            onToggle={() => onToggle("isLit")}
          />

          <EditorViewToggle
            label={"Sun"}
            icon={<WbSunnyIcon />}
            isOn={options.isSunVisible}
            onToggle={() => onToggle("isSunVisible")}
          />

          <EditorViewToggle
            label={"Fog"}
            icon={<FoggyIcon />}
            isOn={options.isFogged}
            onToggle={() => onToggle("isFogged")}
          />

          <EditorToolbarSeparator />

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

          <EditorViewToggle
            label={"Readout"}
            icon={<QueryStatsIcon />}
            isOn={options.isStatsVisible}
            onToggle={() => onToggle("isStatsVisible")}
          />

          {actions ? (
            <>
              <EditorToolbarSeparator />
              {actions}
            </>
          ) : null}
        </>
      }
    />
  );
}
