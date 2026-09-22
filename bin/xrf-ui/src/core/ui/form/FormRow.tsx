import { Typography } from "@mui/material";
import { Nullable, Optional } from "@xrf/types";
import { ReactElement, ReactNode, useId } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** Attributes that associate a control or group with its row's label and visible messages. */
export interface IFormRowControlProps {
  "aria-describedby": Optional<string>;
  "aria-invalid": boolean;
  "aria-labelledby": string;
  id: string;
}

interface IFormRowProps extends BaseComponentProps {
  label: string;
  description?: string;
  isRequired?: boolean;
  error?: Nullable<string>;
  /**
   * What the value currently amounts to, said under the control.
   *
   * Neutral rather than wrong, which is why it is not an `error`: a row reporting that a directory is a game
   * installation is answering the person, not correcting them. An error wins the line when both are present.
   */
  fact?: Nullable<string>;
  /** Overrides the generated ID for a rendered control, or identifies a manually composed control. */
  controlId?: string;
  /**
   * Puts the control beside the label instead of under it.
   */
  isInline?: boolean;
  /** Labels a group through ARIA instead of pointing a native label at one input. */
  isGroup?: boolean;
  /** Supplies accessibility attributes to a control or group, or renders ordinary content. */
  children: ReactNode | ((props: IFormRowControlProps) => ReactNode);
}

/**
 * One labelled row of a form.
 */
export function FormRow({
  "data-testid": dataTestId = "form-row",
  id,
  className,
  label,
  description,
  isRequired = true,
  error,
  fact,
  controlId,
  isInline,
  isGroup = false,
  children,
}: IFormRowProps): ReactElement {
  const generatedId: string = useId();
  const fieldId: string = controlId ?? generatedId;
  const labelId: string = `${fieldId}-label`;
  const message: Nullable<string> = error || fact || null;
  const descriptionId: Optional<string> = description ? `${fieldId}-description` : undefined;
  const messageId: Optional<string> = message ? `${fieldId}-message` : undefined;
  const describedBy: Optional<string> = [descriptionId, messageId].filter(Boolean).join(" ") || undefined;

  const heading: ReactElement = (
    <div className={"min-w-0"}>
      <Typography
        id={labelId}
        className={"block"}
        component={isGroup ? "span" : "label"}
        htmlFor={isGroup ? undefined : typeof children === "function" ? fieldId : controlId}
        variant={"subtitle2"}
      >
        {label}

        {isRequired ? null : (
          <Typography className={"ml-1.5 text-text-secondary"} component={"span"} variant={"caption"}>
            Optional
          </Typography>
        )}
      </Typography>

      {description ? (
        <Typography id={descriptionId} className={"block text-text-secondary"} variant={"caption"}>
          {description}
        </Typography>
      ) : null}
    </div>
  );

  return (
    <div
      data-testid={dataTestId}
      data-form-row={isInline ? "inline" : "stacked"}
      id={id}
      className={cn(isInline ? "flex items-center justify-between gap-4" : "flex flex-col gap-1.5", className)}
    >
      {heading}

      <div className={isInline ? "min-w-0 shrink-0" : "min-w-0"}>
        {typeof children === "function"
          ? children({
              id: fieldId,
              "aria-describedby": describedBy,
              "aria-invalid": Boolean(error),
              "aria-labelledby": labelId,
            })
          : children}
      </div>

      {message ? (
        <Typography id={messageId} className={error ? "text-error" : "text-text-secondary"} variant={"caption"}>
          {message}
        </Typography>
      ) : null}
    </div>
  );
}
