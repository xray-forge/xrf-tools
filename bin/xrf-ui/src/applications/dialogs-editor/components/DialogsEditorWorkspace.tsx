import { Box, MenuItem, TextField } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { DialogPhraseList } from "@/applications/dialogs-editor/components/editor/DialogPhraseList";
import { DialogsTreeMenu } from "@/applications/dialogs-editor/components/editor/DialogsTreeMenu";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { DialogDescriptor, DialogProjectDescriptor } from "@/core/bindings/types/xrf-dialog";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { Nullable } from "@/lib/types/general";

export function DialogsEditorWorkspace(): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  const project: Nullable<DialogProjectDescriptor> = dialogsService.project.value;
  const dialog: Nullable<DialogDescriptor> = dialogsService.dialog.value;
  const languages: Array<string> = dialogsService.languages;

  const onSelect = useCallback(
    (logicalPath: string, id: string) => void dialogsService.selectDialog(logicalPath, id),
    [dialogsService]
  );

  return (
    <Box sx={{ display: "flex", flexGrow: 1, minHeight: 0 }}>
      <DialogsTreeMenu files={project?.files ?? {}} selection={dialogsService.selection} onSelect={onSelect} />

      <Box sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, overflow: "auto" }}>
        {languages.length ? (
          <Box sx={{ borderBottom: "1px solid", borderColor: "divider", padding: 1 }}>
            <TextField
              select
              size={"small"}
              label={"Language"}
              sx={{ minWidth: 180 }}
              value={dialogsService.resolvedLanguage ?? ""}
              onChange={(event) => dialogsService.setLanguage(event.target.value)}
            >
              {languages.map((it: string) => (
                <MenuItem key={it} value={it}>
                  {it}
                </MenuItem>
              ))}
            </TextField>
          </Box>
        ) : null}

        {dialog ? (
          <DialogPhraseList phrases={dialog.phrases} />
        ) : (
          <EmptyState
            title={dialogsService.selection ? "Reading dialog" : "No dialog selected"}
            description={
              dialogsService.selection
                ? "Fetching its phrases."
                : "Pick one from the list to see its phrases and where they lead."
            }
          />
        )}
      </Box>
    </Box>
  );
}
