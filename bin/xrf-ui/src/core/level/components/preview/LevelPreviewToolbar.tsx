import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { default as HexagonIcon } from "@mui/icons-material/Hexagon";
import { default as QueryStatsIcon } from "@mui/icons-material/QueryStats";
import { default as TextureIcon } from "@mui/icons-material/Texture";
import { default as ThreeDRotationIcon } from "@mui/icons-material/ThreeDRotation";
import { ReactElement, ReactNode, useCallback } from "react";

import { LevelBakedAction } from "@/core/level/components/preview/LevelBakedAction";
import { LevelFogAction } from "@/core/level/components/preview/LevelFogAction";
import { LevelLodAction } from "@/core/level/components/preview/LevelLodAction";
import { LevelSunAction } from "@/core/level/components/preview/LevelSunAction";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { ILevelLodOptions } from "@/core/level/lib/lod/level-lod-options";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarSeparator } from "@/core/shell/editor/EditorToolbarSeparator";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewToolbarProps extends BaseComponentProps {
  subtitle?: ReactNode;
  options: ILevelViewOptions;
  /** What the level is lit and fogged with, which the light and fog toggles carry the settings of. */
  lighting: ILevelLighting;
  /** How far trees are drawn in full, which the impostors toggle carries. */
  lod: ILevelLodOptions;
  /** Value pickers the surface contributes, drawn last, as every toolbar in this application orders them. */
  actions?: ReactNode;
  onChangeOptions: (options: ILevelViewOptions) => void;
  onChangeLighting: (lighting: ILevelLighting) => void;
  onChangeLod: (lod: ILevelLodOptions) => void;
  onBack?: () => void;
}

/**
 * Composes the level view toggles in the editor toolbar. A toggle with settings behind it opens them on a click and
 * turns over on a right click.
 */
export function LevelPreviewToolbar({
  "data-testid": dataTestId,
  id,
  className,
  subtitle,
  options,
  lighting,
  lod,
  actions,
  onChangeOptions,
  onChangeLighting,
  onChangeLod,
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

          <LevelLodAction
            isOn={options.isImpostors}
            lod={lod}
            onToggle={() => onToggle("isImpostors")}
            onChange={onChangeLod}
          />

          <EditorToolbarSeparator />

          <LevelBakedAction
            isOn={options.isLit}
            lighting={lighting}
            onToggle={() => onToggle("isLit")}
            onChange={onChangeLighting}
          />

          <LevelSunAction
            isOn={options.isSunVisible}
            lighting={lighting}
            onToggle={() => onToggle("isSunVisible")}
            onChange={onChangeLighting}
          />

          <LevelFogAction
            isOn={options.isFogged}
            lighting={lighting}
            onToggle={() => onToggle("isFogged")}
            onChange={onChangeLighting}
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
