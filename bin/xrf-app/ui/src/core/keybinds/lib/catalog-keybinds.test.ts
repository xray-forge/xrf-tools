import { describe, expect, it } from "@jest/globals";

import { IKeybindCommand } from "@/core/commands";
import { ROOT_KEYBIND_COMMANDS } from "@/core/commands/root-commands";
import { LAUNCHER_KEYBIND_COMMANDS } from "@/core/launcher/commands";
import { IToolDescriptor } from "@/core/routing/tool";
import { TOOL_CATALOG } from "@/ToolCatalog";

import { findChordConflicts, IChordConflict } from "./keymap";

describe("the command catalog", () => {
  it("declares each command id once, so two owners cannot answer to one id", () => {
    const owners: Map<string, Array<string>> = new Map();

    for (const tool of TOOL_CATALOG.tools) {
      for (const command of tool.keybindCommands ?? []) {
        owners.set(command.id, [...(owners.get(command.id) ?? []), tool.id]);
      }
    }

    const shared: Array<[string, Array<string>]> = [...owners.entries()].filter(
      ([, tools]: [string, Array<string>]) => new Set(tools).size > 1
    );

    expect(shared).toEqual([]);
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
