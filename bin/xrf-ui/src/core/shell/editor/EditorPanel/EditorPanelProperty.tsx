import { Typography } from "@mui/material";
import { Fragment, ReactElement, ReactNode } from "react";

import { cn } from "@/lib/dom/dom-name";
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

interface IEditorPanelPropertyProps extends BaseComponentProps {
  label: string;
  value: ReactNode;
  /** Uses the shared identifier and path typography for the value. */
  isMonospace?: boolean;
}

/** Displays a muted label above its full-width value, preserving space for paths and descriptions in narrow panels. */
export function EditorPanelProperty({
  "data-testid": dataTestId = "editor-panel-property",
  id,
  className,
  label,
  value,
  isMonospace = false,
}: IEditorPanelPropertyProps): ReactElement {
  return (
    <dl
      data-testid={dataTestId}
      id={id}
      className={cn("m-0 min-w-0 gap-panel-property-gap py-panel-property leading-panel", className)}
      style={{ display: "flex", flexDirection: "column" }}
    >
      <Typography
        className={"max-w-full leading-panel wrap-anywhere text-text-secondary"}
        component={"dt"}
        variant={"caption"}
      >
        {label}
      </Typography>

      <Typography
        className={cn(
          "m-0 max-w-full leading-panel wrap-anywhere whitespace-pre-wrap",
          isMonospace ? "monospace" : null
        )}
        component={"dd"}
        variant={"body2"}
        style={{ width: "100%", minWidth: 0, textAlign: "start" }}
      >
        {isMonospace && typeof value === "string" ? toBreakableSegments(value) : value}
      </Typography>
    </dl>
  );
}
