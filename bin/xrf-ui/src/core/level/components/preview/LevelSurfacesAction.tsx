import { default as TextureIcon } from "@mui/icons-material/Texture";
import { ReactElement } from "react";

import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelSurfacesActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  onToggle: (option: keyof ILevelViewOptions) => void;
}

/**
 * How the surfaces are drawn: as wireframe, with their textures, with their bumps, under their wall marks.
 */
export function LevelSurfacesAction({
  "data-testid": dataTestId = "level-surfaces-action",
  id,
  className,
  options,
  onToggle,
}: ILevelSurfacesActionProps): ReactElement {
  const { isWireframe, isTextured, isBumped, isWallmarked } = options;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Surfaces"}
      description={[
        isWireframe ? "Wireframe" : "Solid",
        isTextured ? "textured" : "untextured",
        isBumped ? "bumped" : "flat",
        isWallmarked ? "with wall marks" : "without wall marks",
      ].join(", ")}
      icon={<TextureIcon />}
      isActive={isWireframe || isTextured || isBumped || isWallmarked}
    >
      <EditorPopoverGroupSection label={"Wireframe"} isOn={isWireframe} onToggle={() => onToggle("isWireframe")} />

      <EditorPopoverGroupSection label={"Textures"} isOn={isTextured} onToggle={() => onToggle("isTextured")} />

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
