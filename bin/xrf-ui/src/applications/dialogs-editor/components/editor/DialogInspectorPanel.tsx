import { Box, Chip, Divider, Stack, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { DialogInspectorSection } from "@/applications/dialogs-editor/components/editor/DialogInspectorSection";
import { DIALOG_NODE_ID } from "@/applications/dialogs-editor/lib";
import { groupDialogElements, IDialogElementGroup } from "@/applications/dialogs-editor/lib/dialog-elements";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { DialogDescriptor, DialogElementDescriptor, DialogPhraseDescriptor } from "@/core/bindings/types/xrf-dialog";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { Nullable } from "@/lib/types/general";

/** How every identifier in this panel reads: monospace, wrapping, and quieter than the prose above it. */
const IDENTIFIER_SX = {
  color: "text.secondary",
  display: "block",
  fontFamily: "monospace",
  overflowWrap: "anywhere",
} as const;

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

  // Chosen inside the memo rather than beside it: `?? []` mints a new array every render, so a memo
  // keyed on it would never hold and would regroup on every keystroke elsewhere in the application.
  const groups: Array<IDialogElementGroup> = useMemo(() => {
    const elements: ReadonlyArray<DialogElementDescriptor> = isDialogRoot
      ? (dialog?.elements ?? [])
      : (phrase?.elements ?? []);

    return groupDialogElements(elements);
  }, [dialog, isDialogRoot, phrase]);

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

  const badges: Array<ReactElement> = [
    ...(isDialogRoot && dialog.priority !== null
      ? [<Chip key={"priority"} size={"small"} variant={"outlined"} label={`priority ${dialog.priority}`} />]
      : []),
    // Terminality is worded here where there is room for it, and the two spellings read alike because
    // they behave alike: the engine closes the dialog whether a phrase says `is_final` or simply
    // offers nothing. Nearly four in ten phrases are the second kind.
    ...(phrase && (phrase.isFinal || !phrase.next.length)
      ? [<Chip key={"terminal"} size={"small"} variant={"outlined"} label={phrase.isFinal ? "final" : "ends here"} />]
      : []),
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

      <Box sx={{ flexGrow: 1, minHeight: 0, overflowY: "auto", paddingY: 1.5 }}>
        {groups.length ? (
          groups.map((group: IDialogElementGroup, index: number) => (
            <DialogInspectorSection
              key={group.id}
              title={group.title}
              caption={group.caption}
              elements={group.elements}
              isFirst={index === 0}
            />
          ))
        ) : (
          <Typography variant={"body2"} sx={{ color: "text.secondary", paddingX: 2 }}>
            {isDialogRoot ? "This dialog gates nothing." : "This phrase carries no conditions or effects."}
          </Typography>
        )}
      </Box>
    </Box>
  );
}
