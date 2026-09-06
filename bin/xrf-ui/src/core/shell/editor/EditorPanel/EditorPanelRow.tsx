import { Box, Typography } from "@mui/material";
import { Fragment, ReactElement, ReactNode } from "react";

import { MONOSPACE, PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { splitAfterSeparators } from "@/lib/path/separator";

/**
 * A path rendered so it breaks after a separator rather than mid-name.
 *
 * `overflow-wrap` alone breaks wherever the line runs out, which turns `act\actor\act_stalker` into a name split
 * across two lines at no meaningful point. Offering the separators as break opportunities first means a path folds
 * where a reader would fold it; the wrap rule stays behind it for a single name longer than the panel.
 */
function toBreakableSegments(value: string): ReactNode {
    const segments: Array<string> = splitAfterSeparators(value);

    if (segments.length < 2) {
        return value;
    }

    return segments.map((segment: string, index: number) => (
        <Fragment key={index}>
            {segment}
            <wbr />
        </Fragment>
    ));
}

interface IEditorPanelRowProps extends BaseComponentProps {
  label: string;
  value: ReactNode;
  /** Uses the shared identifier and path typography for the value. */
  isMonospace?: boolean;
}

/**
 * One label and value pair, which pairs off or stacks according to the room it is given.
 *
 * Side by side while both halves fit, and the value on a line of its own when it does not, decided by the row rather
 * than by whoever wrote it: the panel is a person's to drag between {@link PANEL.minWidth} and {@link PANEL.maxWidth},
 * so no call site can know which shape it will be asked for. A row of numbers keeps reading as two columns at any
 * usable width, and an engine path stops being three broken characters in a corner.
 */
export function EditorPanelRow({
  "data-testid": dataTestId = "visual-panel-row",
  id,
  className,
  label,
  value,
  isMonospace = false,
}: IEditorPanelRowProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{
        display: "flex",
        flexWrap: "wrap",
        justifyContent: "space-between",
        alignItems: "baseline",
        columnGap: 2,
        paddingY: 0.4,
        minWidth: 0,
        lineHeight: 1.6,
      }}
    >
      <Typography
        variant={"body2"}
        sx={{ color: "text.secondary", flex: "0 0 auto", maxWidth: "100%", overflowWrap: "anywhere" }}
      >
        {label}
      </Typography>

      <Typography
        component={"span"}
        variant={"body2"}
        sx={{
          ...(isMonospace ? MONOSPACE : null),
          // Hugging its content rather than filling the row is what leaves the value where it belongs on both
          // shapes: pushed to the right beside a label, and reading from the left once it is on its own line.
          flex: "0 1 auto",
          minWidth: PANEL.rowValueMinWidth,
          maxWidth: "100%",
          overflowWrap: "anywhere",
        }}
      >
        {isMonospace && typeof value === "string" ? toBreakableSegments(value) : value}
      </Typography>
    </Box>
  );
}
