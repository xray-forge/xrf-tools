import { splitAfterSeparators } from "@xrf/path";
import { Nullable } from "@xrf/types";
import { Fragment, ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { cn } from "#/theme";
import { Link, Typography } from "#/typography";

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

export interface IEditorPanelPropertyProps extends BaseComponentProps {
  label: string;
  value: ReactNode;
  /** What qualifies the value: where a number came from, what a flag switches on. */
  caption?: ReactNode;
  /** Where the value leads outside the application, opened in the system browser. */
  href?: Nullable<string>;
  /** Uses the shared identifier and path typography for the value. */
  isMonospace?: boolean;
  /**
   * Sets the label in a column of its own beside the value once the window is wide enough, as reading surfaces and
   * summaries list their facts. Stacked otherwise, which is what a narrow panel needs.
   */
  isInline?: boolean;
}

/**
 * A muted label and its full-width value: stacked to keep room for paths and descriptions in narrow panels, or side
 * by side in a reading column.
 */
export function EditorPanelProperty({
  "data-testid": dataTestId = "editor-panel-property",
  id,
  className,
  label,
  value,
  caption,
  href = null,
  isMonospace = false,
  isInline = false,
}: IEditorPanelPropertyProps): ReactElement {
  const content: ReactNode = isMonospace && typeof value === "string" ? toBreakableSegments(value) : value;

  return (
    <dl
      data-testid={dataTestId}
      id={id}
      className={cn(
        "m-0 min-w-0 gap-panel-property-gap py-panel-property leading-panel",
        isInline ? "grid grid-cols-1 sm:grid-cols-[var(--container-reading-label)_1fr]" : null,
        className
      )}
      style={isInline ? undefined : { display: "flex", flexDirection: "column" }}
    >
      <Typography
        className={"max-w-full min-w-0 leading-panel wrap-anywhere text-text-secondary"}
        component={"dt"}
        variant={"caption"}
      >
        {label}
      </Typography>

      <dd className={"m-0 max-w-full"} style={{ width: "100%", minWidth: 0, textAlign: "start" }}>
        <Typography
          className={cn("leading-panel wrap-anywhere whitespace-pre-wrap", isMonospace ? "monospace" : null)}
          component={"div"}
          variant={"body2"}
        >
          {href ? (
            <Link href={href} target={"_blank"} rel={"noreferrer"}>
              {content}
            </Link>
          ) : (
            content
          )}
        </Typography>

        {caption ? (
          <Typography className={"block wrap-anywhere text-text-disabled"} component={"div"} variant={"caption"}>
            {caption}
          </Typography>
        ) : null}
      </dd>
    </dl>
  );
}
