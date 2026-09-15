import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Fragment, ReactElement } from "react";

import { EKeybindCommandCategory, IKeybindCommand, KEYBIND_COMMAND_CATEGORY_LABELS } from "@/core/commands";
import { formatChord, parseChord } from "@/core/keybinds";
import { KeymapService } from "@/core/keybinds/services/keymap";
import { MONOSPACE } from "@/core/theme/tokens";
import { StyledComponentProps } from "@/lib/dom/element-types";
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
  sx,
}: StyledComponentProps): Nullable<ReactElement> {
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
    <Box data-testid={dataTestId} id={id} className={className} sx={sx}>
      <Typography variant={"subtitle2"} sx={{ color: "text.primary", marginBottom: 0.5 }}>
        Shortcuts
      </Typography>

      {categories.map((category: EKeybindCommandCategory) => (
        <Fragment key={category}>
          <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block", marginTop: 0.5 }}>
            {KEYBIND_COMMAND_CATEGORY_LABELS[category]}
          </Typography>

          {rows
            .filter((row: IShortcutRow) => row.command.category === category)
            .map(({ command, chords }: IShortcutRow) => (
              <Box
                key={command.id}
                sx={{ display: "flex", alignItems: "baseline", gap: 1, justifyContent: "space-between" }}
              >
                <Typography variant={"body2"} sx={{ lineHeight: 1.55 }}>
                  {command.label}
                </Typography>

                <Typography variant={"caption"} sx={{ ...MONOSPACE, color: "primary.main", whiteSpace: "nowrap" }}>
                  {chords.map((chord: string) => formatChord(parseChord(chord))).join("   ")}
                </Typography>
              </Box>
            ))}
        </Fragment>
      ))}
    </Box>
  );
}
