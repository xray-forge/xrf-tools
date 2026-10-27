import { describe, expect, it, jest } from "@jest/globals";
import { EventBus } from "@wirestate/core";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { mockInvoke } from "@xrf/ipc/testing";
import { ELeaveDecision } from "@xrf/ipc/types/xrf-app-system";

import { EMIT_NOTIFICATION_EVENT } from "@/core/notifications/lib";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import { noop } from "@/lib/callbacks/noop";

import { EditorLifecycleService } from "./editor-lifecycle.service";

describe("EditorLifecycleService", () => {
  it("blocks direct leave requests until every operation releases its own block", () => {
    const { service } = mockInjectedService(EditorLifecycleService);
    const leave = jest.fn();

    service.setBusy("editor", true);
    service.setBusy("form", false);
    service.requestLeave(leave);

    expect(leave).not.toHaveBeenCalled();

    service.setBusy("form", true);
    service.setBusy("editor", false);
    service.requestLeave(leave);

    expect(service.isBusy).toBe(true);
    expect(leave).not.toHaveBeenCalled();

    service.setBusy("form", false);
    service.requestLeave(leave);

    expect(service.isBusy).toBe(false);
    expect(leave).toHaveBeenCalledTimes(1);
  });

  it("keeps the first leave destination and consumes it once", () => {
    const { service } = mockInjectedService(EditorLifecycleService);
    const first = jest.fn();
    const second = jest.fn();

    service.setDraft("document", 2, null);
    service.requestLeave(first);
    service.requestLeave(second);
    service.discardAndLeave();
    service.discardAndLeave();

    expect(first).toHaveBeenCalledTimes(1);
    expect(second).not.toHaveBeenCalled();
  });

  it("refuses another save, navigation, dismissal, or discard while saving", async () => {
    const { service } = mockInjectedService(EditorLifecycleService);
    const answer = deferred<boolean>();
    const save = jest.fn(() => answer.promise);
    const leave = jest.fn();
    const other = jest.fn();

    service.setDraft("document", 1, save);
    service.requestLeave(leave);

    const saving = service.saveAndLeave();

    await service.saveAndLeave();
    service.stay();
    service.discardAndLeave();
    service.requestLeave(other);

    expect(service.isBusy).toBe(true);
    expect(service.isLeavePending).toBe(true);
    expect(save).toHaveBeenCalledTimes(1);
    expect(leave).not.toHaveBeenCalled();
    expect(other).not.toHaveBeenCalled();

    answer.resolve(true);
    await saving;

    expect(service.isBusy).toBe(false);
    expect(service.isLeavePending).toBe(false);
    expect(leave).toHaveBeenCalledTimes(1);
  });

  it.each(["refused", "rejected", "thrown"])(
    "keeps the document and permits retry when saving is %s",
    async (outcome) => {
      const { service, container } = mockInjectedService(EditorLifecycleService);
      const notices: Array<unknown> = [];
      const leave = jest.fn();
      const save = jest.fn(() => {
        if (outcome === "thrown") {
          throw new Error("write failed");
        }

        return outcome === "rejected" ? Promise.reject(new Error("write failed")) : Promise.resolve(false);
      });

      container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));
      service.setDraft("document", 1, save);
      service.requestLeave(leave);
      await service.saveAndLeave();

      expect(service.isSaving).toBe(false);
      expect(service.isLeavePending).toBe(true);
      expect(leave).not.toHaveBeenCalled();
      expect(notices).toHaveLength(outcome === "refused" ? 0 : 1);

      save.mockImplementation(() => Promise.resolve(true));
      await service.saveAndLeave();

      expect(leave).toHaveBeenCalledTimes(1);
    }
  );

  it("ignores a save result after its document was released", async () => {
    const { service } = mockInjectedService(EditorLifecycleService);
    const answer = deferred<boolean>();
    const leave = jest.fn();

    service.setDraft("old", 1, () => answer.promise);
    service.requestLeave(leave);

    const saving = service.saveAndLeave();

    service.releaseDraft("old");
    service.setDraft("new", 3, null);
    answer.resolve(true);
    await saving;

    expect(leave).not.toHaveBeenCalled();
    expect(service.dirtyCount).toBe(3);
    expect(service.isBusy).toBe(false);
    expect(service.isLeavePending).toBe(false);
  });

  it("does not let an old owner's cleanup clear a new document", () => {
    const { service } = mockInjectedService(EditorLifecycleService);
    const leave = jest.fn();

    service.setDraft("old", 1, null);
    service.setDraft("new", 2, null);
    service.requestLeave(leave);
    service.releaseDraft("old");

    expect(service.dirtyCount).toBe(2);
    expect(service.isLeavePending).toBe(true);
  });

  it("tells the backend log each decision, so the log file holds what the page decided beside what the window did", () => {
    const { service } = mockInjectedService(EditorLifecycleService);

    service.setDraft("document", 2, null);
    service.requestExit(noop);
    service.stay();
    service.setBusy("job", true);
    service.requestExit(noop);
    service.exitAnyway();

    const reported: Array<unknown> = mockInvoke.mock.calls
      .filter(([command]) => command === ESystemCommand.REPORT_LEAVE_DECISION)
      .map(([, args]) => args);

    expect(reported).toEqual([
      { decision: ELeaveDecision.EXIT_ASKED, unsaved: 2 },
      { decision: ELeaveDecision.ASKED_ABOUT_UNSAVED, unsaved: 2 },
      { decision: ELeaveDecision.KEPT_OPEN, unsaved: 2 },
      { decision: ELeaveDecision.ASKED_ABOUT_RUNNING_WORK, unsaved: 2 },
      { decision: ELeaveDecision.EXITED_WHILE_BUSY, unsaved: 2 },
    ]);
  });
});
