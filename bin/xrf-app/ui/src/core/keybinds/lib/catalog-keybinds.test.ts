import { describe, expect, it } from "@jest/globals";

import { IKeybindCommand } from "@/core/commands";
import { ROOT_KEYBIND_COMMANDS } from "@/core/container/root-commands";
import { LAUNCHER_KEYBIND_COMMANDS } from "@/core/launcher/commands";
import { IToolDescriptor } from "@/core/routing/tool";
import { TOOL_CATALOG } from "@/ToolCatalog";

import { findChordConflicts, IChordConflict } from "./keymap";

describe("the command catalog", () => {
  it("defines each command id once, so two definitions cannot answer to one id", () => {
    // A domain's command listed by every tool that shows the domain is one definition reached from several tools.
    const definitions: Map<string, Set<IKeybindCommand>> = new Map();

    for (const command of [
      ...ROOT_KEYBIND_COMMANDS,
      ...LAUNCHER_KEYBIND_COMMANDS,
      ...TOOL_CATALOG.tools.flatMap((tool: IToolDescriptor) => tool.keybindCommands ?? []),
    ]) {
      definitions.set(command.id, (definitions.get(command.id) ?? new Set()).add(command));
    }

    const redefined: Array<string> = [...definitions.entries()]
      .filter(([, commands]: [string, Set<IKeybindCommand>]) => commands.size > 1)
      .map(([id]: [string, Set<IKeybindCommand>]) => id);

    expect(redefined).toEqual([]);
  });

  it("binds no chord to two commands reachable at once", () => {
    const contested: Array<[string, Array<IChordConflict>]> = [];

    for (const tool of TOOL_CATALOG.tools) {
      const reachable: Array<IKeybindCommand> = [...ROOT_KEYBIND_COMMANDS, ...(tool.keybindCommands ?? [])];
      const conflicts: Array<IChordConflict> = findChordConflicts(reachable);

      if (conflicts.length > 0) {
        contested.push([tool.id, conflicts]);
      }
    }

    expect(contested).toEqual([]);
  });

  it("has a conflict-free root set, which every tool inherits", () => {
    expect(findChordConflicts(ROOT_KEYBIND_COMMANDS)).toEqual([]);
  });

  it("audits the home screen, which is reachable without being a tool", () => {
    expect(findChordConflicts([...ROOT_KEYBIND_COMMANDS, ...LAUNCHER_KEYBIND_COMMANDS])).toEqual([]);
  });

  it("lists only commands whose chords parse, since a malformed one never fires", () => {
    const everyCommand: Array<IKeybindCommand> = [
      ...ROOT_KEYBIND_COMMANDS,
      ...LAUNCHER_KEYBIND_COMMANDS,
      ...TOOL_CATALOG.tools.flatMap((tool: IToolDescriptor) => tool.keybindCommands ?? []),
    ];

    // `findChordConflicts` parses every chord it is given, so a malformed one throws rather than reporting.
    expect(() => findChordConflicts(everyCommand)).not.toThrow();
  });
});
