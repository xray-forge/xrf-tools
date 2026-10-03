import { default as TextureIcon } from "@mui/icons-material/Texture";
import { ReactElement } from "react";

import { ELevelShading, getLevelShading, LEVEL_SHADINGS } from "@/core/level/lib/view/level-shading";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { ChoiceListFormRow, IChoiceFormRowOption } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** Every shading as the list offers it. */
const SHADING_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ELevelShading>> = LEVEL_SHADINGS.map(({ label, value }) => ({
  label,
  value,
}));

interface ILevelShadingActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  /** What the viewport shows of its surfaces. */
  shading: ELevelShading;
  onToggle: (option: keyof ILevelViewOptions) => void;
  onChangeShading: (shading: ELevelShading) => void;
}

/**
 * How the level is shaded: the frame as dressed, as clay or by shader, or one of the targets it is built from; and the
 * surfaces as wireframe, with their bumps, under their wall marks, whichever of those is shown.
 */
export function LevelShadingAction({
  "data-testid": dataTestId = "level-shading-action",
  id,
  className,
  options,
  shading,
  onToggle,
  onChangeShading,
}: ILevelShadingActionProps): ReactElement {
  const { isWireframe, isBumped, isWallmarked } = options;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Shading"}
      description={[
        getLevelShading(shading).label,
        isWireframe ? "wireframe" : null,
        isBumped ? null : "flat",
        isWallmarked ? null : "without wall marks",
      ]
        .filter((it): it is string => it !== null)
        .join(", ")}
      icon={<TextureIcon />}
      isActive={shading !== ELevelShading.FINAL || isWireframe || !isBumped || !isWallmarked}
    >
      <ChoiceListFormRow
        data-testid={"level-shading"}
        label={"Show"}
        description={"The frame as dressed, as clay, by shader, or one of the targets it is built from"}
        options={SHADING_OPTIONS}
        value={shading}
        filterFrom={SHADING_OPTIONS.length + 1}
        onChange={onChangeShading}
      />

      <EditorPopoverGroupSection label={"Wireframe"} isOn={isWireframe} onToggle={() => onToggle("isWireframe")} />

      <EditorPopoverGroupSection
        label={"Bumps"}
        description={isBumped ? "Shaded with the bump pairs their textures declare" : "Shaded flat, as though unbound"}
        isOn={isBumped}
        onToggle={() => onToggle("isBumped")}
      />

      <EditorPopoverGroupSection
        label={"Wall marks"}
        description={isWallmarked ? "The decals compiled into the level, over what they mark" : "The surfaces bare"}
        isOn={isWallmarked}
        onToggle={() => onToggle("isWallmarked")}
      />
    </EditorPopoverGroup>
  );
}
