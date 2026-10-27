import { formatPercent } from "@xrf/format";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { TuneIcon } from "@xrf/ui/icons";
import { Slider, Typography } from "@xrf/ui/material";
import { ReactElement, useCallback } from "react";

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
  const onSlideDetail = useCallback(
    (_: Event, value: number | Array<number>) => {
      if (typeof value === "number") {
        onChange(1 - value / 100);
      }
    },
    [onChange]
  );

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
      <div className={"w-toolbar-slider px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Mesh detail
        </Typography>

        <Slider
          aria-label={"Mesh detail"}
          size={"small"}
          min={0}
          max={100}
          value={Math.round((1 - detail) * 100)}
          valueLabelDisplay={"auto"}
          valueLabelFormat={(value: number) => `${value}%`}
          onChange={onSlideDetail}
        />
      </div>
    </EditorPopoverAction>
  );
}
