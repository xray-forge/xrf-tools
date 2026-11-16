import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";
import { flow } from "@wirestate/mobx";
import { TFlow } from "@xrf/flow";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { CommandReporter } from "@/core/operations/lib/command-reporter";
import { mockContainer } from "@/fixtures/utils/container";
import { IWrittenFileNouns, WrittenFileActions } from "@/tools/level-viewer/lib/files/written-file-actions";

interface IFile {
  path: string;
}

function setup(nouns: IWrittenFileNouns = { name: "recording" }) {
  const bus: EventBus = mockContainer().get(EventBus);
  const raised: Array<INotificationPayload> = [];
  const state: { file: Nullable<IFile> } = { file: { path: "C:\\out\\a.mp4" } };
  const actions: WrittenFileActions<IFile> = new WrittenFileActions(
    new CommandReporter(new Logger("test"), bus, "level-viewer"),
    { nouns, current: () => state.file, toPath: ({ path }) => path }
  );

  bus.subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
    raised.push(event.payload as INotificationPayload)
  );

  return { actions, raised, state };
}

function run<T>(action: () => TFlow<T>): Promise<T> {
  return flow(action)();
}

describe("WrittenFileActions", () => {
  const writeText = jest.fn<(text: string) => Promise<void>>();

  beforeEach(() => {
    writeText.mockReset().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText } });
  });

  it("deletes the current file, answering whether it did", async () => {
    const { actions, raised } = setup();
    const command = jest.fn<(file: IFile) => Promise<void>>().mockResolvedValue(undefined);

    await expect(run(() => actions.remove(command))).resolves.toBe(true);
    expect(command).toHaveBeenCalledWith({ path: "C:\\out\\a.mp4" });

    command.mockRejectedValue(new Error("locked"));

    await expect(run(() => actions.remove(command))).resolves.toBe(false);
    expect(raised).toEqual([
      {
        details: "C:\\out\\a.mp4\nlocked",
        severity: ENotificationSeverity.ERROR,
        source: "level-viewer",
        title: "Could not delete the recording",
      },
    ]);
  });

  it("does nothing where there is no file", async () => {
    const { actions, raised, state } = setup();
    const command = jest.fn<(file: IFile) => Promise<void>>();

    state.file = null;

    await expect(run(() => actions.remove(command))).resolves.toBe(false);
    await run(() => actions.copyPath());

    expect(command).not.toHaveBeenCalled();
    expect(writeText).not.toHaveBeenCalled();
    expect(raised).toEqual([]);
  });

  it("copies the file's path, naming the file in the failure as its notice noun reads", async () => {
    const { actions, raised } = setup({ name: "screenshot report", notice: "screenshot's report" });

    await run(() => actions.copyPath());

    expect(writeText).toHaveBeenCalledWith("C:\\out\\a.mp4");

    writeText.mockRejectedValue(new Error("denied"));

    await run(() => actions.copyPath());

    expect(raised).toEqual([
      {
        details: "C:\\out\\a.mp4\ndenied",
        severity: ENotificationSeverity.ERROR,
        source: "level-viewer",
        title: "Could not copy the screenshot's report's path",
      },
    ]);
  });
});
