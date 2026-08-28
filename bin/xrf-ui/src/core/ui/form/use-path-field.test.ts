import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { open } from "@tauri-apps/plugin-dialog";
import { act, renderHook } from "@testing-library/react";
import { StrictMode } from "react";

import { EApplicationId } from "@/core/routing/application";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Nullable } from "@/lib/types/general";

describe("usePathField", () => {
  const STORAGE_KEY: string = "xrf.form.archives-packer.source";

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
});
