import { default as FilterCenterFocusIcon } from "@mui/icons-material/FilterCenterFocus";
import { default as GrainIcon } from "@mui/icons-material/Grain";
import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { default as HexagonIcon } from "@mui/icons-material/Hexagon";
import { default as LightbulbIcon } from "@mui/icons-material/Lightbulb";
import { default as TextureIcon } from "@mui/icons-material/Texture";
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
  /** Whether any surface of the open level is modulated by a detail texture. */
  hasDetail?: boolean;
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
  hasDetail = true,
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

          <EditorViewToggle
            label={"Detail"}
            icon={<GrainIcon />}
            isOn={options.isDetailed}
            isDisabled={!hasDetail}
            unavailableTitle={"No surface of this level is detailed"}
            onToggle={() => onToggle("isDetailed")}
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

          <EditorToolbarSeparator />

          <EditorViewToggle
            label={"Grid"}
            icon={<GridOnIcon />}
            isOn={options.isGridVisible}
            onToggle={() => onToggle("isGridVisible")}
          />

          <EditorViewToggle
            label={"Origin"}
            icon={<FilterCenterFocusIcon />}
            isOn={options.isAxesVisible}
            onToggle={() => onToggle("isAxesVisible")}
          />
        </>
      }
    />
  );
}
