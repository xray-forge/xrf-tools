import { formatNumber } from "@xrf/format";
import { ReactElement } from "react";

import { ELevelLookFieldOff } from "@/applications/level-viewer/lib/look/level-look-field-off";
import { ILevelLookField } from "@/applications/level-viewer/lib/look/level-look-fields";
import { ILevelLook } from "@/core/level/lib/look/level-look";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";

export interface ILevelLookSlidersProps {
  /** The sliders, each a number of the look. */
  fields: ReadonlyArray<ILevelLookField>;
  /** The look they read and edit. */
  look: ILevelLook;
  /** Whether the light is physical, which leaves the engine's own tonemap off. */
  isPhysical: boolean;
  /** Whether the physical light stands at real-world levels, which leave the game's flat ambient off too. */
  isPhysicalLevels: boolean;
  /** Takes the look a slider edited. */
  onEdit: (look: ILevelLook) => void;
}

/** A run of the look's sliders, each off where what it sets is not read. */
export function LevelLookSliders({
  fields,
  look,
  isPhysical,
  isPhysicalLevels,
  onEdit,
}: ILevelLookSlidersProps): ReactElement {
  function isOff(field: ILevelLookField): boolean {
    switch (field.offUnder) {
      case ELevelLookFieldOff.PHYSICAL_LIGHT:
        return isPhysical;
      case ELevelLookFieldOff.PHYSICAL_LEVELS:
        return isPhysicalLevels;
      default:
        return false;
    }
  }

  return (
    <div className={"flex flex-col gap-1"}>
      {fields.map((field: ILevelLookField) => (
        <RenderValueSlider
          key={field.label}
          label={field.label}
          value={field.read(look)}
          min={field.min}
          max={field.max}
          step={field.step}
          isDisabled={isOff(field)}
          format={(value: number) => formatNumber(value, 2)}
          onChange={(value: number) => onEdit(field.write(look, value))}
        />
      ))}
    </div>
  );
}
