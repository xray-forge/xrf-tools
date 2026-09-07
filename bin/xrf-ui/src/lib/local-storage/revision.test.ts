import { beforeEach, describe, expect, it, jest } from "@jest/globals";

import { setLocalStorageValue, setLocalStorageValueSafe } from "@/lib/local-storage/read-write";
import { getLocalStorageRevision, subscribeToLocalStorage } from "@/lib/local-storage/revision";

describe("local storage revision", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("advances on a write, so a reader can tell that something changed", () => {
    const before: number = getLocalStorageRevision();

    setLocalStorageValue("xrf.form.a.b", "C:\\gamedata");

    expect(getLocalStorageRevision()).toBeGreaterThan(before);
  });

  it("advances on a removal too, because a cleared key changes what is stored", () => {
    setLocalStorageValue("xrf.form.a.b", "C:\\gamedata");

    const before: number = getLocalStorageRevision();

    setLocalStorageValue("xrf.form.a.b", null);

    expect(getLocalStorageRevision()).toBeGreaterThan(before);
  });

  it("tells its watchers, which is the wiring between writing and reporting", () => {
    const onChange = jest.fn();
    const stop: () => void = subscribeToLocalStorage(onChange);

    try {
      // Both write paths reach the counter: the failsafe one delegates rather than writing on its own.
      setLocalStorageValue("xrf.form.a.b", "C:\\gamedata");
      setLocalStorageValueSafe("xrf.form-recents.a.b", "[]");

      expect(onChange).toHaveBeenCalledTimes(2);
    } finally {
      stop();
    }
  });

  it("stops telling a watcher that unsubscribed", () => {
    const onChange = jest.fn();

    subscribeToLocalStorage(onChange)();

    setLocalStorageValue("xrf.form.a.b", "C:\\gamedata");

    expect(onChange).not.toHaveBeenCalled();
  });

  it("says nothing when storage is not there to write to", () => {
    const onChange = jest.fn();
    const stop: () => void = subscribeToLocalStorage(onChange);

    try {
      jest.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
        throw new DOMException("exceeded the quota", "QuotaExceededError");
      });

      // A refused write changed nothing, so a reader recomputing on this would be recomputing for no reason.
      expect(() => setLocalStorageValueSafe("xrf.form-recents.a.b", "[]")).not.toThrow();
      expect(onChange).not.toHaveBeenCalled();
    } finally {
      stop();
      jest.restoreAllMocks();
    }
  });
});
