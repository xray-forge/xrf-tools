import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { Nullable } from "@/lib/types/general";

interface IFormRowProps {
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
  /** Ties the label to the control it names. Without it the field reads as unlabelled. */
  controlId?: string;
  /**
   * Puts the control beside the label instead of under it.
   */
  isInline?: boolean;
  children: ReactNode;
}

/**
 * One labelled row of a form.
 */
export function FormRow({
  label,
  description,
  isRequired = true,
  error,
  fact,
  controlId,
  isInline,
  children,
}: IFormRowProps): ReactElement {
  const heading: ReactElement = (
    <Box sx={{ minWidth: 0 }}>
      <Typography component={"label"} htmlFor={controlId} variant={"subtitle2"} sx={{ display: "block" }}>
        {label}

        {isRequired ? null : (
          <Typography component={"span"} variant={"caption"} sx={{ marginLeft: 0.75, color: "text.secondary" }}>
            Optional
          </Typography>
        )}
      </Typography>

      {description ? (
        <Typography variant={"caption"} sx={{ display: "block", color: "text.secondary" }}>
          {description}
        </Typography>
      ) : null}
    </Box>
  );

  return (
    <Box
      sx={
        isInline
          ? { display: "flex", alignItems: "center", justifyContent: "space-between", gap: 2 }
          : { display: "flex", flexDirection: "column", gap: 0.75 }
      }
    >
      {heading}

      <Box sx={{ minWidth: 0, flexShrink: isInline ? 0 : undefined }}>{children}</Box>

      {error || fact ? (
        <Typography variant={"caption"} sx={{ color: error ? "error.main" : "text.secondary" }}>
          {error ?? fact}
        </Typography>
      ) : null}
    </Box>
  );
}
