import { PointerEventHandler, ReactElement, ReactNode } from "react";

import { CheckIcon } from "#/icons/check";
import { ListItemText } from "#/material/list";
import { MenuItem } from "#/material/menu";
import { Tooltip } from "#/material/tooltip";
import { Typography } from "#/material/typography";

import { IMenubarItem } from "./menubar-menu";

interface IMenubarRowProps {
  item: IMenubarItem;
  onChoose: (item: IMenubarItem) => void;
  onPointerEnter?: PointerEventHandler<HTMLElement>;
}

/** One row of an open menu: its icon or a switch's mark, the label, its shortcut at the end, its description beside. */
export function MenubarRow({ item, onChoose, onPointerEnter }: IMenubarRowProps): ReactElement {
  const isSwitch: boolean = item.isChecked !== undefined;
  // One of exclusive choices is marked by a dot, as a radio button is; any other switch by a check.
  const mark: ReactNode = item.isExclusive ? <span className={"size-1.5 rounded-full bg-current"} /> : <CheckIcon />;

  return (
    <Tooltip describeChild disableInteractive placement={"right"} title={item.description ?? ""}>
      <MenuItem
        aria-checked={isSwitch ? item.isChecked : undefined}
        className={"mx-1 min-h-7 gap-2 rounded-control px-2 py-0.5"}
        dense={true}
        disabled={item.isDisabled}
        role={isSwitch ? (item.isExclusive ? "menuitemradio" : "menuitemcheckbox") : "menuitem"}
        onClick={() => onChoose(item)}
        onPointerEnter={onPointerEnter}
      >
        <span aria-hidden={true} className={"flex w-4 shrink-0 justify-center text-text-secondary [&>svg]:text-base"}>
          {item.isChecked ? mark : null}
          {isSwitch ? null : item.icon}
        </span>

        <ListItemText className={"my-0"} primary={item.label} slotProps={{ primary: { variant: "body2" } }} />

        {item.shortcut ? (
          <Typography className={"ml-auto shrink-0 pl-8 text-text-secondary"} variant={"caption"}>
            {item.shortcut}
          </Typography>
        ) : null}
      </MenuItem>
    </Tooltip>
  );
}
