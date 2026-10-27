import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { autorun, isComputedProp } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { clearLocalStorage, getLocalStorageValue, setLocalStorageValue } from "@/core/storage/read-write";
import { StoredValue } from "@/core/storage/stored-value";
import { IStoredValueCodec, jsonCodec, textCodec } from "@/core/storage/stored-value-codec";

const KEY: string = "xrf.preference.fixture";

interface IFixturePoint {
  x: number;
}

const POINT_CODEC: IStoredValueCodec<Nullable<IFixturePoint>> = jsonCodec((parsed: unknown) =>
  typeof (parsed as Nullable<IFixturePoint>)?.x === "number" ? (parsed as IFixturePoint) : null
);

const TEXT_CODEC: IStoredValueCodec<string> = textCodec((stored: Nullable<string>) => stored ?? "none");

describe("StoredValue", () => {
  beforeEach(() => {
    clearLocalStorage();
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it("reads what the key holds once, when built", () => {
    setLocalStorageValue(KEY, "30");

    const stored: StoredValue<number> = new StoredValue(KEY, textCodec(Number));

    setLocalStorageValue(KEY, "60");

    expect(stored.value).toBe(30);
    expect(isComputedProp(stored, "value")).toBe(true);
  });

  it("reads the codec's default where nothing is stored", () => {
    expect(new StoredValue(KEY, TEXT_CODEC).value).toBe("none");
  });

  it("holds and writes each value set, observably", () => {
    const stored: StoredValue<string> = new StoredValue(KEY, TEXT_CODEC);
    const seen: Array<string> = [];
    const stop: () => void = autorun(() => seen.push(stored.value));

    stored.set("b");
    stop();

    expect(seen).toEqual(["none", "b"]);
    expect(getLocalStorageValue(KEY)).toBe("b");
  });

  it("drops a value that does not read for the default", () => {
    setLocalStorageValue(KEY, "{not json");

    expect(new StoredValue(KEY, POINT_CODEC).value).toBeNull();
  });

  it("keeps JSON, a null value clearing the key", () => {
    const stored: StoredValue<Nullable<IFixturePoint>> = new StoredValue(KEY, POINT_CODEC);

    stored.set({ x: 4 });
    expect(new StoredValue(KEY, POINT_CODEC).value).toEqual({ x: 4 });

    stored.set(null);
    expect(getLocalStorageValue(KEY)).toBeNull();
  });

  it("logs a refused write rather than throwing it, holding the value set", () => {
    jest.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("QuotaExceededError");
    });

    const stored: StoredValue<string> = new StoredValue(KEY, TEXT_CODEC);

    expect(() => stored.set("b")).not.toThrow();
    expect(stored.value).toBe("b");
  });
});
