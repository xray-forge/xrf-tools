import { Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Fragment, ReactElement } from "react";

import { EKeybindCommandCategory, IKeybindCommand, KEYBIND_COMMAND_CATEGORY_LABELS } from "@/core/commands";
import { formatChord, parseChord } from "@/core/keybinds";
import { KeymapService } from "@/core/keybinds/services/keymap";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** One command and the chords it currently answers to. */
interface IShortcutRow {
  command: IKeybindCommand;
  chords: ReadonlyArray<string>;
}

/**
 * The keyboard, for the tool that is open.
 */
export function ApplicationHelpShortcuts({
  "data-testid": dataTestId = "application-help-shortcuts",
  id,
  className,
}: BaseComponentProps): Nullable<ReactElement> {
  const keymapService: KeymapService = useInjection(KeymapService);

  const rows: Array<IShortcutRow> = keymapService.commands
    .map((command: IKeybindCommand) => ({ chords: keymapService.getChords(command), command }))
    .filter((row: IShortcutRow) => row.chords.length > 0);

  if (rows.length === 0) {
    return null;
  }

  const categories: Array<EKeybindCommandCategory> = (
    Object.keys(KEYBIND_COMMAND_CATEGORY_LABELS) as Array<EKeybindCommandCategory>
  ).filter((category: EKeybindCommandCategory) => rows.some((row: IShortcutRow) => row.command.category === category));

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <Typography className={"mb-1 text-text-primary"} variant={"subtitle2"}>
        Shortcuts
      </Typography>

      {categories.map((category: EKeybindCommandCategory) => (
        <Fragment key={category}>
          <Typography className={"mt-1 block text-text-secondary"} variant={"caption"}>
            {KEYBIND_COMMAND_CATEGORY_LABELS[category]}
          </Typography>

          {rows
            .filter((row: IShortcutRow) => row.command.category === category)
            .map(({ command, chords }: IShortcutRow) => (
              <div key={command.id} className={"flex items-baseline justify-between gap-2"}>
                <Typography variant={"body2"} sx={{ lineHeight: 1.55 }}>
                  {command.label}
                </Typography>

                <Typography className={"monospace whitespace-nowrap text-primary"} variant={"caption"}>
                  {chords.map((chord: string) => formatChord(parseChord(chord))).join("   ")}
                </Typography>
              </div>
            ))}
        </Fragment>
      ))}
    </div>
  );
}
