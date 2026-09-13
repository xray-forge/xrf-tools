import { describe, expect, it } from "@jest/globals";

import { APPLICATION_CATALOG } from "@/ApplicationCatalog";
import { ICommandDescriptor } from "@/core/commands";
import { ROOT_COMMANDS } from "@/core/commands/root-commands";
import { findChordConflicts, IChordConflict } from "@/core/keybinds/lib/keymap";
import { IApplicationDescriptor } from "@/core/routing/application";

describe("the command catalog", () => {
  it("declares each command id once, so two owners cannot answer to one id", () => {
    const owners: Map<string, Array<string>> = new Map();

    for (const application of APPLICATION_CATALOG.applications) {
      for (const command of application.commands ?? []) {
        owners.set(command.id, [...(owners.get(command.id) ?? []), application.id]);
      }
    }

    const shared: Array<[string, Array<string>]> = [...owners.entries()].filter(
      ([, applications]: [string, Array<string>]) => new Set(applications).size > 1
    );

    expect(shared).toEqual([]);
  });

  it("binds no chord to two commands reachable at once", () => {
    const contested: Array<[string, Array<IChordConflict>]> = [];

    for (const application of APPLICATION_CATALOG.applications) {
      const reachable: Array<ICommandDescriptor> = [...ROOT_COMMANDS, ...(application.commands ?? [])];
      const conflicts: Array<IChordConflict> = findChordConflicts(reachable);

      if (conflicts.length > 0) {
        contested.push([application.id, conflicts]);
      }
    }

    expect(contested).toEqual([]);
  });

  it("has a conflict-free root set, which every application inherits", () => {
    expect(findChordConflicts(ROOT_COMMANDS)).toEqual([]);
  });

  it("lists only commands whose chords parse, since a malformed one never fires", () => {
    const everyCommand: Array<ICommandDescriptor> = [
      ...ROOT_COMMANDS,
      ...APPLICATION_CATALOG.applications.flatMap((application: IApplicationDescriptor) => application.commands ?? []),
    ];

    // `findChordConflicts` parses every chord it is given, so a malformed one throws rather than reporting.
    expect(() => findChordConflicts(everyCommand)).not.toThrow();
  });
});
