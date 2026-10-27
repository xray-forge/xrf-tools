import { ReactElement } from "react";

import { CheckIcon } from "#/icons";
import { ListItemText, MenuItem, Typography } from "#/material";

import { IMenubarItem } from "./menubar-menu";

interface IMenubarRowProps {
  item: IMenubarItem;
  onChoose: (item: IMenubarItem) => void;
}

/** One row of an open menu: its icon or a switch's check, the label, and its shortcut at the end. */
export function MenubarRow({ item, onChoose }: IMenubarRowProps): ReactElement {
  const isSwitch: boolean = item.isChecked !== undefined;

  return (
    <MenuItem
      aria-checked={isSwitch ? item.isChecked : undefined}
      className={"mx-1 min-h-7 gap-2 rounded-control px-2 py-0.5"}
      dense={true}
      disabled={item.isDisabled}
      role={isSwitch ? "menuitemcheckbox" : "menuitem"}
      onClick={() => onChoose(item)}
    >
      <span aria-hidden={true} className={"flex w-4 shrink-0 justify-center text-text-secondary [&>svg]:text-base"}>
        {item.isChecked ? <CheckIcon /> : null}
        {isSwitch ? null : item.icon}
      </span>

      <ListItemText className={"my-0"} primary={item.label} slotProps={{ primary: { variant: "body2" } }} />

      {item.shortcut ? (
        <Typography className={"ml-auto shrink-0 pl-8 text-text-secondary"} variant={"caption"}>
          {item.shortcut}
        </Typography>
      ) : null}
    </MenuItem>
  );
}
