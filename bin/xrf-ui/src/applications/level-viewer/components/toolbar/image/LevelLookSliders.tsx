import { formatNumber } from "@xrf/format";
import { ReactElement } from "react";

import { ILevelLookField } from "@/applications/level-viewer/lib/look/level-look-fields";
import { ILevelLook } from "@/core/level/lib/look/level-look";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";

export interface ILevelLookSlidersProps {
  /** The sliders, each a number of the look. */
  fields: ReadonlyArray<ILevelLookField>;
  /** The look they read and edit. */
  look: ILevelLook;
  /** Whether the light is physical, which leaves the engine's own sliders off. */
  isPhysical: boolean;
  /** Takes the look a slider edited. */
  onEdit: (look: ILevelLook) => void;
}

/** A run of the look's sliders, the engine's own off under the physical light. */
export function LevelLookSliders({ fields, look, isPhysical, onEdit }: ILevelLookSlidersProps): ReactElement {
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
          isDisabled={field.isEngineOnly === true && isPhysical}
          format={(value: number) => formatNumber(value, 2)}
          onChange={(value: number) => onEdit(field.write(look, value))}
        />
      ))}
    </div>
  );
}
