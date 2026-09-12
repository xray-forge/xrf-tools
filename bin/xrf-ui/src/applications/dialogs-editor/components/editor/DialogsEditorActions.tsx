import { default as TranslateIcon } from "@mui/icons-material/Translate";
import { Menu, MenuItem } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { MouseEvent, ReactElement, useCallback, useId, useState } from "react";

import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * Which language the phrase lines are read in.
 */
export function DialogsEditorActions({
  "data-testid": dataTestId = "dialogs-editor-actions",
  id,
  className,
}: BaseComponentProps): Nullable<ReactElement> {
  const dialogsService: DialogsService = useInjection(DialogsService);

  const generatedActionId: string = useId();
  const actionId: string = id ?? generatedActionId;
  const menuId: string = useId();

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
      <EditorIconAction
        data-testid={dataTestId}
        id={actionId}
        className={className}
        label={"Change language"}
        description={`Language: ${selected ?? "none"}`}
        icon={<TranslateIcon />}
        aria-haspopup={"menu"}
        aria-expanded={Boolean(anchor)}
        aria-controls={anchor ? menuId : undefined}
        onClick={onOpen}
      />

      <Menu
        open={Boolean(anchor)}
        anchorEl={anchor}
        slotProps={{ list: { id: menuId, "aria-labelledby": actionId } }}
        onClose={onClose}
      >
        {languages.map((it: string) => (
          <MenuItem key={it} selected={it === selected} onClick={() => onSelect(it)}>
            {it}
          </MenuItem>
        ))}
      </Menu>
    </>
  );
}
