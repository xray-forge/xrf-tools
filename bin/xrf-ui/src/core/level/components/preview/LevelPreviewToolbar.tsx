import { default as HexagonIcon } from "@mui/icons-material/Hexagon";
import { default as PaletteIcon } from "@mui/icons-material/Palette";
import { default as TextureIcon } from "@mui/icons-material/Texture";
import { ReactElement, ReactNode, useCallback } from "react";

import { ILevelSurfaceOptions } from "@/core/level/lib/level-surface-material";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewToolbarProps extends BaseComponentProps {
  subtitle?: ReactNode;
  options: ILevelSurfaceOptions;
  onChangeOptions: (options: ILevelSurfaceOptions) => void;
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
  onChangeOptions,
  onBack,
}: ILevelPreviewToolbarProps): ReactElement {
  const onToggle = useCallback(
    (option: keyof ILevelSurfaceOptions) => {
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
            label={"Surface colours"}
            icon={<PaletteIcon />}
            isOn={options.isSurfaceColored}
            onToggle={() => onToggle("isSurfaceColored")}
          />
        </>
      }
    />
  );
}
