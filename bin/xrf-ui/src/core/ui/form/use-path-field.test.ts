import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { open, save } from "@tauri-apps/plugin-dialog";
import { exists } from "@tauri-apps/plugin-fs";
import { act, renderHook } from "@testing-library/react";
import { StrictMode } from "react";

import { EApplicationId } from "@/core/routing/application";
import { IPathRecord } from "@/core/ui/form/path-recents";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Nullable } from "@/lib/types/general";

describe("usePathField", () => {
  const STORAGE_KEY: string = "xrf.form.archives-packer.source";

  /** The remembered paths of a field, newest first. */
  function recentPaths(field: IPathField): Array<string> {
    return field.recents.records.map((it: IPathRecord) => it.path);
  }

  function renderField(seed?: () => Promise<Nullable<string>>) {
    return renderHook(() => usePathField({ application: EApplicationId.ARCHIVES_PACKER, id: "source", seed }), {
      wrapper: StrictMode,
    });
  }

  interface IDeferred<T> {
    promise: Promise<T>;
    resolve: (value: T) => void;
    reject: (reason: unknown) => void;
  }

  /** A guess whose completion the test decides, which is what every race here is about. */
  function deferred<T>(): IDeferred<T> {
    let resolve!: (value: T) => void;
    let reject!: (reason: unknown) => void;

    const promise: Promise<T> = new Promise<T>((resolveIt, rejectIt) => {
      resolve = resolveIt;
      reject = rejectIt;
    });

    return { promise, resolve, reject };
  }

  beforeEach(() => {
    window.localStorage.clear();
    jest.mocked(open).mockResolvedValue(null);
    jest.mocked(save).mockResolvedValue(null);
    // Restated rather than left to the module factory: `clearMocks` clears calls but keeps implementations, so a test
    // that says a path is absent would otherwise say it for every test after it.
    jest.mocked(exists).mockResolvedValue(true);
  });

  it("restores the remembered path on the first render", () => {
    window.localStorage.setItem(STORAGE_KEY, "C:\\projects\\stored");

    const { result } = renderField();

    expect(result.current.value).toBe("C:\\projects\\stored");
  });

  it("writes nothing while only mounting and remounting", () => {
    window.localStorage.setItem(STORAGE_KEY, "C:\\projects\\stored");

    const setItem = jest.spyOn(Storage.prototype, "setItem");
    const removeItem = jest.spyOn(Storage.prototype, "removeItem");

    try {
      renderField().unmount();
      renderField().unmount();

      expect(setItem).not.toHaveBeenCalled();
      expect(removeItem).not.toHaveBeenCalled();
      expect(window.localStorage.getItem(STORAGE_KEY)).toBe("C:\\projects\\stored");
    } finally {
      setItem.mockRestore();
      removeItem.mockRestore();
    }
  });

  it("remembers what the dialog returned", async () => {
    jest.mocked(open).mockResolvedValue("C:\\projects\\picked");

    const { result } = renderField();

    await act(() => result.current.select());

    expect(result.current.value).toBe("C:\\projects\\picked");
    expect(window.localStorage.getItem(STORAGE_KEY)).toBe("C:\\projects\\picked");
  });

  it("leaves the remembered path alone when the dialog is cancelled", async () => {
    window.localStorage.setItem(STORAGE_KEY, "C:\\projects\\stored");

    const { result } = renderField();

    await act(() => result.current.select());

    expect(result.current.value).toBe("C:\\projects\\stored");
    expect(window.localStorage.getItem(STORAGE_KEY)).toBe("C:\\projects\\stored");
  });

  it("remembers a path a caller sets and forgets a cleared one", () => {
    const { result }: { result: { current: IPathField } } = renderField();

    act(() => result.current.setValue("C:\\projects\\typed"));

    expect(window.localStorage.getItem(STORAGE_KEY)).toBe("C:\\projects\\typed");

    act(() => result.current.clear());

    expect(result.current.value).toBeNull();
    expect(window.localStorage.getItem(STORAGE_KEY)).toBeNull();
  });

  it("fills from the seed without remembering it, and only when nothing was remembered", async () => {
    const seed = jest.fn<() => Promise<Nullable<string>>>(async () => "C:\\projects\\seeded");

    const { result } = renderField(seed);

    await act(async () => undefined);

    // Counted only as "it asked": a guess is superseded rather than deduplicated, so the contract is what lands in
    // the field and what reaches storage, never how many times it was asked for.
    expect(seed).toHaveBeenCalled();
    expect(result.current.value).toBe("C:\\projects\\seeded");
    expect(window.localStorage.getItem(STORAGE_KEY)).toBeNull();

    window.localStorage.setItem(STORAGE_KEY, "C:\\projects\\stored");
    seed.mockClear();

    const stored = renderField(seed);

    await act(async () => undefined);

    expect(seed).not.toHaveBeenCalled();
    expect(stored.result.current.value).toBe("C:\\projects\\stored");
  });
  it("refuses a guess that resolves after the user picked a path", async () => {
    const pending: IDeferred<Nullable<string>> = deferred<Nullable<string>>();

    jest.mocked(open).mockResolvedValue("C:\\projects\\picked");

    const { result } = renderField(() => pending.promise);

    await act(() => result.current.select());

    expect(result.current.value).toBe("C:\\projects\\picked");

    await act(async () => {
      pending.resolve("C:\\projects\\seeded");

      await pending.promise;
    });

    expect(result.current.value).toBe("C:\\projects\\picked");
    expect(window.localStorage.getItem(STORAGE_KEY)).toBe("C:\\projects\\picked");
  });

  it("refuses a guess that resolves after a value was set", async () => {
    const pending: IDeferred<Nullable<string>> = deferred<Nullable<string>>();

    const { result } = renderField(() => pending.promise);

    act(() => result.current.setValue("C:\\projects\\typed"));

    await act(async () => {
      pending.resolve("C:\\projects\\seeded");

      await pending.promise;
    });

    expect(result.current.value).toBe("C:\\projects\\typed");
  });

  it("leaves a picked path alone when the guess fails", async () => {
    const pending: IDeferred<Nullable<string>> = deferred<Nullable<string>>();

    jest.mocked(open).mockResolvedValue("C:\\projects\\picked");

    const { result } = renderField(() => pending.promise);

    await act(() => result.current.select());

    await act(async () => {
      pending.reject(new Error("nothing to guess"));

      await pending.promise.catch(() => undefined);
    });

    expect(result.current.value).toBe("C:\\projects\\picked");
  });

  it("writes nothing when a guess resolves after unmount", async () => {
    const pending: IDeferred<Nullable<string>> = deferred<Nullable<string>>();

    const { result, unmount } = renderField(() => pending.promise);

    unmount();

    await act(async () => {
      pending.resolve("C:\\projects\\seeded");

      await pending.promise;
    });

    expect(result.current.value).toBeNull();
  });

  it("asks for the guess again when the field is cleared", async () => {
    const seed = jest.fn<() => Promise<Nullable<string>>>(async () => "C:\\projects\\seeded");

    const { result } = renderField(seed);

    await act(async () => undefined);

    expect(result.current.value).toBe("C:\\projects\\seeded");

    act(() => result.current.setValue("C:\\projects\\typed"));

    expect(result.current.value).toBe("C:\\projects\\typed");

    const asked: number = seed.mock.calls.length;

    await act(async () => result.current.clear());

    expect(seed.mock.calls.length).toBeGreaterThan(asked);
    expect(result.current.value).toBe("C:\\projects\\seeded");
    expect(window.localStorage.getItem(STORAGE_KEY)).toBeNull();
  });

  it("asks for the guess again when the dialog is cancelled before the first one landed", async () => {
    const pending: IDeferred<Nullable<string>> = deferred<Nullable<string>>();
    const seed = jest.fn<() => Promise<Nullable<string>>>();

    seed.mockReturnValueOnce(pending.promise).mockResolvedValue("C:\\projects\\seeded");

    const { result } = renderField(seed);
    const asked: number = seed.mock.calls.length;

    await act(() => result.current.select());

    expect(seed.mock.calls.length).toBeGreaterThan(asked);

    await act(async () => undefined);

    expect(result.current.value).toBe("C:\\projects\\seeded");
  });

  describe("where the dialog opens", () => {
    function renderSaveField() {
      return renderHook(
        () => usePathField({ application: EApplicationId.ARCHIVES_PACKER, id: "source", isSave: true }),
        { wrapper: StrictMode }
      );
    }

    it("opens at the path the field is holding", async () => {
      window.localStorage.setItem(STORAGE_KEY, "C:\\projects\\stored");
      jest.mocked(exists).mockResolvedValue(true);

      const { result } = renderField();

      await act(() => result.current.select());

      expect(open).toHaveBeenCalledWith(expect.objectContaining({ defaultPath: "C:\\projects\\stored" }));
    });

    it("opens at the directory above a path that is no longer there", async () => {
      window.localStorage.setItem(STORAGE_KEY, "C:\\projects\\gone\\all.spawn");
      // Absent itself, present one level up: the directory it would have been in is still the right place to look.
      jest.mocked(exists).mockImplementation(async (path: unknown) => path === "C:\\projects\\gone");

      const { result } = renderField();

      await act(() => result.current.select());

      expect(open).toHaveBeenCalledWith(expect.objectContaining({ defaultPath: "C:\\projects\\gone" }));
    });

    it("leaves it to the host when the field leads nowhere", async () => {
      window.localStorage.setItem(STORAGE_KEY, "Q:\\nothing\\here");
      jest.mocked(exists).mockResolvedValue(false);

      const { result } = renderField();

      await act(() => result.current.select());

      expect(open).toHaveBeenCalledWith(expect.objectContaining({ defaultPath: undefined }));
    });

    it("leaves it to the host for an empty field", async () => {
      const { result } = renderField();

      await act(() => result.current.select());

      expect(open).toHaveBeenCalledWith(expect.objectContaining({ defaultPath: undefined }));
    });

    it("offers a destination back whole, so the name chosen for it survives", async () => {
      window.localStorage.setItem(STORAGE_KEY, "C:\\output\\packed.db");
      jest.mocked(exists).mockImplementation(async (path: unknown) => path === "C:\\output");

      const { result } = renderSaveField();

      await act(() => result.current.select());

      // A destination need not exist. Falling back to its directory would throw away the file name.
      expect(save).toHaveBeenCalledWith(expect.objectContaining({ defaultPath: "C:\\output\\packed.db" }));
    });
  });

  describe("history", () => {
    const RECENTS_KEY: string = "xrf.form-recents.archives-packer.source";

    it("records nothing for a typed path, however much is typed", () => {
      const { result } = renderField();

      // The field is written on every keystroke, so this is what a path being typed looks like. Recording any of it
      // would remember six prefixes of one directory, which is the whole reason typing is excluded.
      act(() => result.current.setValue("C"));
      act(() => result.current.setValue("C:"));
      act(() => result.current.setValue("C:\\pro"));
      act(() => result.current.setValue("C:\\projects"));

      expect(result.current.recents.records).toHaveLength(0);
      expect(window.localStorage.getItem(RECENTS_KEY)).toBeNull();
    });

    it("records what the dialog returned", async () => {
      jest.mocked(open).mockResolvedValue("C:\\projects\\picked");

      const { result } = renderField();

      await act(() => result.current.select());

      expect(recentPaths(result.current)).toEqual(["C:\\projects\\picked"]);
      expect(window.localStorage.getItem(RECENTS_KEY)).toContain("C:\\\\projects\\\\picked");
    });

    it("records the value when the form it is in was submitted", () => {
      const { result } = renderField();

      act(() => result.current.setValue("C:\\projects\\typed"));
      act(() => result.current.commit());

      expect(recentPaths(result.current)).toEqual(["C:\\projects\\typed"]);
    });

    it("records nothing on submission when the field is empty", () => {
      const { result } = renderField();

      act(() => result.current.commit());

      expect(result.current.recents.records).toHaveLength(0);
    });

    it("keeps the history when the field is cleared, because clearing is not forgetting", () => {
      window.localStorage.setItem(STORAGE_KEY, "C:\\projects\\stored");

      const { result } = renderField();

      act(() => result.current.commit());
      act(() => result.current.clear());

      expect(result.current.value).toBeNull();
      expect(window.localStorage.getItem(STORAGE_KEY)).toBeNull();
      expect(recentPaths(result.current)).toEqual(["C:\\projects\\stored"]);
    });

    it("offers nothing for a value that was never picked or run here", async () => {
      window.localStorage.setItem(STORAGE_KEY, "C:\\projects\\stored");

      const { result } = renderField(async () => "C:\\projects\\seeded");

      await act(async () => undefined);

      // The history holds what was used, never what happens to be in the field. Standing a value in for an empty
      // history made an entry appear for a stored path and not for a seeded one, which are indistinguishable on
      // screen.
      expect(result.current.recents.records).toHaveLength(0);
      expect(window.localStorage.getItem(RECENTS_KEY)).toBeNull();
    });

    it("records a seeded path only once it has been run", async () => {
      const { result } = renderField(async () => "C:\\projects\\seeded");

      await act(async () => undefined);

      expect(result.current.value).toBe("C:\\projects\\seeded");
      expect(result.current.recents.records).toHaveLength(0);

      act(() => result.current.commit());

      expect(recentPaths(result.current)).toEqual(["C:\\projects\\seeded"]);
    });

    it("takes a path from the history the way it takes one from the dialog", () => {
      const { result } = renderField();

      act(() => result.current.setValue("C:\\projects\\first"));
      act(() => result.current.commit());
      act(() => result.current.setValue("C:\\projects\\second"));
      act(() => result.current.commit());

      act(() => result.current.recents.pick("C:\\projects\\first"));

      expect(result.current.value).toBe("C:\\projects\\first");
      expect(window.localStorage.getItem(STORAGE_KEY)).toBe("C:\\projects\\first");
      expect(recentPaths(result.current)).toEqual(["C:\\projects\\first", "C:\\projects\\second"]);
    });

    it("forgets one entry without touching the others or the field", () => {
      const { result } = renderField();

      act(() => result.current.setValue("C:\\projects\\first"));
      act(() => result.current.commit());
      act(() => result.current.setValue("C:\\projects\\second"));
      act(() => result.current.commit());

      act(() => result.current.recents.forget("C:\\projects\\first"));

      expect(result.current.value).toBe("C:\\projects\\second");
      expect(recentPaths(result.current)).toEqual(["C:\\projects\\second"]);
    });
  });
});
