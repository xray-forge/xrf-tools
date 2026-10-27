import { describe, expect, it, jest } from "@jest/globals";
import { autorun } from "@wirestate/mobx";
import { SHELL_SESSION_TOOL } from "@xrf/ipc/session";

import { EToolId } from "@/core/routing/tool";

import { ShellContributions } from "./shell-contributions";

const FIRST: EToolId = EToolId.TRANSLATIONS_EDITOR;
const SECOND: EToolId = EToolId.DIALOGS_EDITOR;

describe("ShellContributions", () => {
  it("draws each tool's values only for that tool", () => {
    const contributions: ShellContributions<string> = new ShellContributions();

    contributions.publish("shell", SHELL_SESSION_TOOL, ["notifications"]);
    contributions.publish("first", FIRST, ["files"]);

    expect(contributions.get(FIRST)).toEqual(["files"]);
    expect(contributions.get(SECOND)).toEqual([]);
    expect(contributions.get(SHELL_SESSION_TOOL)).toEqual(["notifications"]);
  });

  it("keeps many owners at once, each in the place it first published at", () => {
    const contributions: ShellContributions<string> = new ShellContributions();

    contributions.publish("layout", FIRST, ["layout"]);
    contributions.publish("viewport", FIRST, ["viewport-v1"]);
    contributions.publish("capture", FIRST, ["capture"]);
    contributions.publish("viewport", FIRST, ["viewport-v2", "viewport-extra"]);
    contributions.publish("other", SECOND, ["other"]);

    expect(contributions.get(FIRST)).toEqual(["layout", "viewport-v2", "viewport-extra", "capture"]);

    contributions.release("viewport");

    expect(contributions.get(FIRST)).toEqual(["layout", "capture"]);
    expect(contributions.get(SECOND)).toEqual(["other"]);
  });

  it("does not let an outgoing owner's release erase what the next one published", () => {
    const contributions: ShellContributions<string> = new ShellContributions();

    contributions.publish("old", FIRST, ["old"]);
    contributions.publish("new", SECOND, ["current"]);
    contributions.release("old");

    expect(contributions.get(SECOND)).toEqual(["current"]);
    expect(contributions.get(FIRST)).toEqual([]);
  });

  it("changes nothing observed when an owner repeats values that draw alike for the same tool", () => {
    const contributions: ShellContributions<string> = new ShellContributions(
      (first: string, second: string) => first.toLowerCase() === second.toLowerCase()
    );
    const onRead = jest.fn();

    contributions.publish("owner", FIRST, ["3 files"]);

    const dispose = autorun(() => onRead(contributions.get(FIRST)));

    contributions.publish("owner", FIRST, ["3 FILES"]);
    contributions.release("absent");

    expect(onRead).toHaveBeenCalledTimes(1);

    contributions.publish("owner", SECOND, ["3 files"]);

    expect(onRead).toHaveBeenCalledTimes(2);
    expect(contributions.get(FIRST)).toEqual([]);

    dispose();
  });

  it("copies the caller's array and keeps the values themselves as published", () => {
    const contributions: ShellContributions<{ render: () => null }> = new ShellContributions();
    const value: { render: () => null } = { render: jest.fn(() => null) };
    const values: Array<{ render: () => null }> = [value];

    contributions.publish("owner", FIRST, values);
    values.length = 0;

    expect(contributions.get(FIRST)).toHaveLength(1);
    expect(contributions.get(FIRST)[0]).toBe(value);
    expect(value.render).not.toHaveBeenCalled();
  });

  it("forgets every owner when cleared", () => {
    const contributions: ShellContributions<string> = new ShellContributions();

    contributions.publish("shell", SHELL_SESSION_TOOL, ["notifications"]);
    contributions.publish("first", FIRST, ["files"]);
    contributions.clear();

    expect(contributions.get(FIRST)).toEqual([]);
    expect(contributions.get(SHELL_SESSION_TOOL)).toEqual([]);
  });
});
