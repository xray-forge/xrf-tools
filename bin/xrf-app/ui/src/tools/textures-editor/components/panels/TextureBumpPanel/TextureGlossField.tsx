import { Nullable } from "@xrf/types";
import { EditorPopoverNote } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";

interface ITextureGlossFieldProps extends BaseComponentProps {
  value: number;
  /** The mean gloss below which the backend reports a dark surface, or null before its vocabulary arrives. */
  minimum: Nullable<number>;
  isDisabled?: boolean;
  onChange: (value: number) => void;
}

/** A gloss level as the slider reads it. */
function formatGloss(value: number): string {
  return value.toFixed(2);
}

/**
 * One gloss level for the whole surface, for a texture authored without a mask.
 *
 * The warning below the SDK's own threshold is shown before the run rather than after it, because it is knowable from
 * this number alone: a constant gloss is its own mean. A mask has to be read before anyone can say the same, which is
 * why that case is reported in the outcome instead.
 */
export function TextureGlossField({
  "data-testid": dataTestId = "texture-gloss-field",
  id,
  className,
  value,
  minimum,
  isDisabled = false,
  onChange,
}: ITextureGlossFieldProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderValueSlider
        label={"Gloss level"}
        description={"The one level the whole surface takes, without a mask"}
        value={value}
        min={0}
        max={1}
        step={0.05}
        format={formatGloss}
        isDisabled={isDisabled}
        onChange={onChange}
      />

      {minimum !== null && value < minimum ? (
        <EditorPopoverNote tone={"warning"}>
          {`Below ${minimum}, the surface shows almost no specular response. The pair is still written.`}
        </EditorPopoverNote>
      ) : null}
    </div>
  );
}
