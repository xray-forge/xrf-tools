import { default as TranslateIcon } from "@mui/icons-material/Translate";
import { IconButton, Menu, MenuItem, Tooltip } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { MouseEvent, ReactElement, useCallback, useState } from "react";

import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { Nullable } from "@/lib/types/general";

/**
 * Which language the phrase lines are read in.
 */
export function DialogsEditorActions(): Nullable<ReactElement> {
  const dialogsService: DialogsService = useInjection(DialogsService);

  const [anchor, setAnchor] = useState<Nullable<HTMLElement>>(null);

  const languages: Array<string> = dialogsService.languages;
  const selected: Nullable<string> = dialogsService.resolvedLanguage;

  const onOpen = useCallback((event: MouseEvent<HTMLElement>) => setAnchor(event.currentTarget), []);

  const onClose = useCallback(() => setAnchor(null), []);

  const onSelect = useCallback(
    (language: string) => {
      dialogsService.setLanguage(language);
      setAnchor(null);
    },
    [dialogsService]
  );

  if (!languages.length) {
    return null;
  }

  return (
    <>
      <Tooltip title={`Language: ${selected ?? "none"}`}>
        <IconButton aria-label={"Change language"} onClick={onOpen}>
          <TranslateIcon />
        </IconButton>
      </Tooltip>

      <Menu open={Boolean(anchor)} anchorEl={anchor} onClose={onClose}>
        {languages.map((it: string) => (
          <MenuItem key={it} selected={it === selected} onClick={() => onSelect(it)}>
            {it}
          </MenuItem>
        ))}
      </Menu>
    </>
  );
}
