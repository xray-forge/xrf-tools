import { default as TuneIcon } from "@mui/icons-material/Tune";
import { Box, Slider, Typography } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IVisualMeshDetailProps extends BaseComponentProps {
  /** Position in each submesh's collapse chain: 0 is full detail, 1 is coarsest. */
  detail: number;
  hasDetailLevels: boolean;
  onChange: (detail: number) => void;
}

/**
 * Mesh quality control for the model's continuous edge-collapse chain.
 *
 * Detail is a slider rather than a list of levels because an X-Ray slide-window table is one entry per edge collapse:
 * a measured character submesh carries 948 of them, so there is nothing to enumerate. Moving it costs a draw range
 * and nothing else, since every level is already in the uploaded index buffer. A model with nothing to decimate shows
 * the control disabled rather than hidden, so the toolbar does not change shape as the user steps through a tree.
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
      description={hasDetailLevels ? `Mesh detail: ${Math.round((1 - detail) * 100)}%` : "Nothing to decimate"}
      icon={<TuneIcon />}
      isDisabled={!hasDetailLevels}
      isHighlighted={detail !== 0 && hasDetailLevels}
    >
      <Box sx={{ paddingX: 2, paddingY: 1, width: LAYOUT.toolbarSliderWidth }}>
        <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
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
      </Box>
    </EditorPopoverAction>
  );
}
