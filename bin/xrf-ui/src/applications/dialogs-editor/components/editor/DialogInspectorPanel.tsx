import { Box, Chip, Divider, Stack, Typography } from "@mui/material";
import { GridColDef } from "@mui/x-data-grid";
import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { DIALOG_NODE_ID } from "@/applications/dialogs-editor/lib";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { DialogDescriptor, DialogElementDescriptor, DialogPhraseDescriptor } from "@/core/bindings/types/xrf-dialog";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { DataTable } from "@/core/ui/table";
import { identifierColumn, textColumn } from "@/core/ui/table/columns";
import { Nullable } from "@/lib/types/general";

/** One element as the property list shows it. Repeated keys stay repeated rows. */
interface IInspectedElement {
  /** Row identity, which the element name cannot supply because names repeat. */
  index: number;
  name: string;
  value: string;
}

/**
 * The columns of the property list.
 *
 * Values are monospaced because most of them are identifiers the engine resolves — an info portion, a
 * phrase id, a script reference. Not syntax-highlighted: a dotted reference such as
 * `xr_conditions.actor_has_pda` holds no keyword, string or number for a grammar to colour, so
 * highlighting it would cost a pass over every row and render the same characters.
 */
const COLUMNS: Array<GridColDef> = [textColumn("name", "Element", 150), identifierColumn("value", "Value", 260)];

/**
 * Element kinds the surrounding surface already shows, so the property list does not repeat them.
 *
 * `text` is the line in this panel's own header, and `next` is drawn on the canvas as a numbered edge.
 * A deny-list rather than the node badges' allow-list, deliberately: badges are a glance and admit
 * only behaviour, where this is the full record and should omit nothing it is not already showing.
 * `script_text` therefore stays — the header has no line to show for a scripted phrase, so the
 * reference is the only thing that identifies it.
 */
const SHOWN_ELSEWHERE: ReadonlySet<DialogElementDescriptor["kind"]> = new Set<DialogElementDescriptor["kind"]>([
  "next",
  "text",
]);

/** How every identifier in this panel reads: monospace, wrapping, and quieter than the prose above it. */
const IDENTIFIER_SX = {
  color: "text.secondary",
  display: "block",
  fontFamily: "monospace",
  overflowWrap: "anywhere",
} as const;

function toRows(elements: ReadonlyArray<DialogElementDescriptor>): Array<IInspectedElement> {
  return elements
    .filter((element: DialogElementDescriptor) => !SHOWN_ELSEWHERE.has(element.kind))
    .map((element: DialogElementDescriptor, index: number) => ({
      index,
      name: element.name,
      value: element.value,
    }));
}

/**
 * What one node of the open dialog carries.
 */
export function DialogInspectorPanel(): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  const dialog: Nullable<DialogDescriptor> = dialogsService.dialog.value;
  const nodeId: Nullable<string> = dialogsService.inspectedNodeId;
  const isDialogRoot: boolean = nodeId === DIALOG_NODE_ID;

  const phrase: Nullable<DialogPhraseDescriptor> = useMemo(
    () => dialog?.phrases.find((it: DialogPhraseDescriptor) => it.id === nodeId) ?? null,
    [dialog, nodeId]
  );

  if (!dialog || !nodeId) {
    return (
      <EmptyState
        title={"Nothing selected"}
        description={"Pick a node on the canvas to see the text and conditions it carries."}
      />
    );
  }

  // A node the dialog no longer holds. Reachable while a language switch is in flight, because the
  // canvas keeps its selection across the re-fetch.
  if (!isDialogRoot && !phrase) {
    return <EmptyState title={"Phrase is gone"} description={`This dialog no longer declares '${nodeId}'.`} />;
  }

  const elements: ReadonlyArray<DialogElementDescriptor> = isDialogRoot ? dialog.elements : (phrase?.elements ?? []);
  const badges: Array<ReactElement> = [
    ...(isDialogRoot && dialog.priority !== null
      ? [<Chip key={"priority"} size={"small"} variant={"outlined"} label={`priority ${dialog.priority}`} />]
      : []),
    ...(phrase?.isFinal ? [<Chip key={"final"} size={"small"} variant={"outlined"} label={"final"} />] : []),
    ...(phrase?.textKey && !phrase.text
      ? [<Chip key={"untranslated"} size={"small"} color={"warning"} variant={"outlined"} label={"untranslated"} />]
      : []),
  ];

  return (
    <Box sx={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
      <Box sx={{ padding: 2 }}>
        <Typography variant={"subtitle2"}>{isDialogRoot ? "Dialog" : "Phrase"}</Typography>

        <Typography variant={"caption"} sx={{ ...IDENTIFIER_SX, marginTop: 0.25 }}>
          {isDialogRoot ? dialog.id : phrase?.id}
        </Typography>

        {isDialogRoot ? (
          <Typography variant={"body2"} sx={{ color: "text.secondary", marginTop: 1 }}>
            {dialog.phrases.length} {dialog.phrases.length === 1 ? "phrase" : "phrases"}
            {dialog.language ? ` · ${dialog.language}` : ""}
          </Typography>
        ) : (
          <>
            <Typography variant={"body2"} sx={{ marginTop: 1, overflowWrap: "anywhere" }}>
              {phrase?.text ?? "No text for this language."}
            </Typography>

            {phrase?.textKey ? (
              <Typography variant={"caption"} sx={{ ...IDENTIFIER_SX, marginTop: 0.5 }}>
                {phrase.textKey}
              </Typography>
            ) : null}
          </>
        )}

        {badges.length ? (
          <Stack direction={"row"} spacing={0.75} sx={{ flexWrap: "wrap", marginTop: 1.25, rowGap: 0.75 }}>
            {badges}
          </Stack>
        ) : null}
      </Box>

      <Divider />

      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          flexGrow: 1,
          minHeight: 0,
          paddingBottom: 2,
          paddingTop: 1.5,
          paddingX: 2,
        }}
      >
        <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
          {isDialogRoot ? "Conditions" : "Elements"}
        </Typography>

        <Box sx={{ display: "flex", flexGrow: 1, marginTop: 1, minHeight: 0 }}>
          <DataTable
            rows={toRows(elements)}
            columns={COLUMNS}
            getRowId={(row: IInspectedElement) => row.index}
            emptyLabel={isDialogRoot ? "This dialog carries no conditions." : "This phrase carries no elements."}
            countNoun={"element"}
          />
        </Box>
      </Box>
    </Box>
  );
}
