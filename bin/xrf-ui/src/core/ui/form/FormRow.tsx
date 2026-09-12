import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode, useId } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable, Optional } from "@/lib/types/general";

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
    <Box sx={{ minWidth: 0 }}>
      <Typography
        id={labelId}
        component={isGroup ? "span" : "label"}
        htmlFor={isGroup ? undefined : typeof children === "function" ? fieldId : controlId}
        variant={"subtitle2"}
        sx={{ display: "block" }}
      >
        {label}

        {isRequired ? null : (
          <Typography component={"span"} variant={"caption"} sx={{ marginLeft: 0.75, color: "text.secondary" }}>
            Optional
          </Typography>
        )}
      </Typography>

      {description ? (
        <Typography id={descriptionId} variant={"caption"} sx={{ display: "block", color: "text.secondary" }}>
          {description}
        </Typography>
      ) : null}
    </Box>
  );

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={
        isInline
          ? { display: "flex", alignItems: "center", justifyContent: "space-between", gap: 2 }
          : { display: "flex", flexDirection: "column", gap: 0.75 }
      }
    >
      {heading}

      <Box sx={{ minWidth: 0, flexShrink: isInline ? 0 : undefined }}>
        {typeof children === "function"
          ? children({
              id: fieldId,
              "aria-describedby": describedBy,
              "aria-invalid": Boolean(error),
              "aria-labelledby": labelId,
            })
          : children}
      </Box>

      {message ? (
        <Typography id={messageId} variant={"caption"} sx={{ color: error ? "error.main" : "text.secondary" }}>
          {message}
        </Typography>
      ) : null}
    </Box>
  );
}
