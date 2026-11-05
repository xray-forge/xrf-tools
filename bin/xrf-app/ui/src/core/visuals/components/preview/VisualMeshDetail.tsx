import { formatPercent } from "@xrf/format";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { EditorPopoverContent } from "@xrf/ui/editor/popover-content";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { TuneIcon } from "@xrf/ui/icons/tune";
import { ReactElement, useCallback } from "react";

import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";

interface IVisualMeshDetailProps extends BaseComponentProps {
  /** Position in each submesh's collapse chain: 0 is full detail, 1 is coarsest. */
  detail: number;
  hasDetailLevels: boolean;
  onChange: (detail: number) => void;
}

/**
 * Mesh quality control for the model's continuous edge-collapse chain.
 */
export function VisualMeshDetail({
  "data-testid": dataTestId = "visual-mesh-detail",
  id,
  className,
  detail,
  hasDetailLevels,
  onChange,
}: IVisualMeshDetailProps): ReactElement {
  // Quality increases to the right, while the stored collapse fraction decreases.
  const onChangeQuality = useCallback((quality: number) => onChange(1 - quality / 100), [onChange]);

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Mesh detail"}
      description={hasDetailLevels ? `Mesh detail: ${formatPercent(1 - detail)}` : "Nothing to decimate"}
      icon={<TuneIcon />}
      isDisabled={!hasDetailLevels}
      isActive={detail !== 0 && hasDetailLevels}
    >
      <EditorPopoverContent label={"Mesh detail"}>
        <RenderValueSlider
          label={"Quality"}
          description={"How much of each submesh's collapse chain is kept, the full mesh at 100%."}
          value={Math.round((1 - detail) * 100)}
          min={0}
          max={100}
          step={1}
          format={(value: number) => `${value}%`}
          onChange={onChangeQuality}
        />
      </EditorPopoverContent>
    </EditorPopoverAction>
  );
}
