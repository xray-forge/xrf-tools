import { Box, Chip, Stack, Typography } from "@mui/material";
import { ReactElement } from "react";

import { DialogElementDescriptor, DialogPhraseDescriptor } from "@/core/bindings/types/xrf-dialog";

/** Element kinds a phrase carries that are worth a badge; the rest read as plain rows below. */
const BADGE_KINDS: ReadonlySet<DialogElementDescriptor["kind"]> = new Set<DialogElementDescriptor["kind"]>([
  "action",
  "disableInfo",
  "dontHasInfo",
  "giveInfo",
  "hasInfo",
  "isFinal",
  "precondition",
  "scriptText",
]);

export interface IDialogPhraseListProps {
  phrases: ReadonlyArray<DialogPhraseDescriptor>;
}

/**
 * Every phrase of the open dialog, in document order.
 *
 * A list until the canvas lands, showing what the canvas will draw: the resolved line, the id, the
 * tags a phrase carries, and where it can go next. `next` order is presentation order, so the
 * sequence is numbered rather than left to look incidental.
 */
export function DialogPhraseList({ phrases }: IDialogPhraseListProps): ReactElement {
  return (
    <Stack spacing={1} sx={{ padding: 2 }}>
      {phrases.map((phrase: DialogPhraseDescriptor) => (
        <Box
          key={phrase.id}
          sx={{
            backgroundColor: "background.paper",
            border: "1px solid",
            borderColor: "divider",
            borderRadius: 1,
            padding: 1.5,
          }}
        >
          <Stack direction={"row"} spacing={1} sx={{ alignItems: "baseline", flexWrap: "wrap" }}>
            <Typography variant={"caption"} sx={{ color: "text.secondary", fontFamily: "monospace" }}>
              {phrase.id}
            </Typography>

            <Typography
              variant={"body2"}
              sx={{
                color: phrase.text ? "text.primary" : "text.secondary",
                fontStyle: phrase.text ? "normal" : "italic",
              }}
            >
              {phrase.text ?? phrase.textKey ?? "(built from script)"}
            </Typography>
          </Stack>

          <Stack direction={"row"} spacing={0.5} sx={{ flexWrap: "wrap", marginTop: 0.5 }}>
            {phrase.textKey && !phrase.text ? (
              <Chip size={"small"} color={"warning"} variant={"outlined"} label={"untranslated"} />
            ) : null}

            {phrase.elements
              .filter((element: DialogElementDescriptor) => BADGE_KINDS.has(element.kind))
              .map((element: DialogElementDescriptor, index: number) => (
                <Chip key={`${element.name}-${index}`} size={"small"} variant={"outlined"} label={element.name} />
              ))}

            {phrase.next.map((next: string, index: number) => (
              <Chip
                key={`${next}-${index}`}
                size={"small"}
                color={"primary"}
                variant={"outlined"}
                label={`${index + 1} → ${next}`}
              />
            ))}
          </Stack>
        </Box>
      ))}
    </Stack>
  );
}
