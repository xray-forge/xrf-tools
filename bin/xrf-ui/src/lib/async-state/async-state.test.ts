import { describe, expect, it, jest } from "@jest/globals";

import { AsyncState, EAsyncStatus } from "@/lib/async-state";

describe("AsyncState", () => {
  it("projects retained data through loading and failure without changing the source", () => {
    const payload = { name: "project" };
    const snapshot = { sessionId: "session", value: payload };
    const ready = AsyncState.ready(snapshot);
    const loading = ready.asLoading();
    const error = new Error("refresh failed");
    const failed = loading.asFailed(error);

    for (const source of [AsyncState.idle(snapshot), ready, loading, failed]) {
      const projected = source.map((value) => value.value);

      expect(projected.value).toBe(payload);
      expect(projected.status).toBe(source.status);
      expect(projected.error).toBe(source.error);
      expect(source.value).toBe(snapshot);
    }

    expect(ready.isReady).toBe(true);
    expect(ready.error).toBeNull();
  });

  it("preserves empty states without invoking the projection", () => {
    const project = jest.fn((value: number) => value.toString());
    const states = [
      AsyncState.idle<number>(),
      AsyncState.loading<number>(),
      AsyncState.ready<number>(null),
      AsyncState.failed<number>(new Error("failed")),
    ];

    for (const source of states) {
      const projected = source.map(project);

      expect(projected.value).toBeNull();
      expect(projected.status).toBe(source.status);
      expect(projected.error).toBe(source.error);
    }

    expect(project).not.toHaveBeenCalled();
  });

  it.each([0, false, ""])("projects an available falsy value: %s", (value) => {
    expect(AsyncState.ready(value).map((item) => ({ item })).value).toEqual({ item: value });
  });

  it.each([{ value: null }, { value: [] }, { value: 0 }, { value: false }, { value: "" }])(
    "distinguishes idle from a successful empty value: $value",
    ({ value }) => {
      const idle = AsyncState.idle(value);
      const ready = idle.asReady();

      expect(idle.status).toBe(EAsyncStatus.IDLE);
      expect(idle.isIdle).toBe(true);
      expect(idle.isReady).toBe(false);
      expect(ready.status).toBe(EAsyncStatus.READY);
      expect(ready.isReady).toBe(true);
      expect(ready.isIdle).toBe(false);
      expect(ready.isLoading).toBe(false);
      expect(ready.isFailed).toBe(false);
      expect(ready.error).toBeNull();
      expect(ready.value).toBe(value);
    }
  );

  it("keeps successful content through a failed refresh and retry without changing earlier states", () => {
    const value = { name: "previous" };
    const error = new Error("refresh failed");
    const ready = AsyncState.ready(value);
    const loading = ready.asLoading();
    const failed = loading.asFailed(error);
    const retry = failed.asLoading();
    const completed = retry.asReady(null);

    expect(ready.isReady).toBe(true);
    expect(loading.isLoading).toBe(true);
    expect(loading.error).toBeNull();
    expect(loading.value).toBe(value);
    expect(failed.isFailed).toBe(true);
    expect(failed.isLoading).toBe(false);
    expect(failed.error).toBe(error);
    expect(failed.value).toBe(value);
    expect(retry.isLoading).toBe(true);
    expect(retry.error).toBeNull();
    expect(retry.value).toBe(value);
    expect(completed.isReady).toBe(true);
    expect(completed.value).toBeNull();
  });

  it("resets a failed resource to idle and permits an explicit fallback", () => {
    const failed = AsyncState.failed(new Error("failed"), [1]);
    const reset = failed.asIdle();
    const seeded = failed.asIdle([]);

    expect(reset.isIdle).toBe(true);
    expect(reset.value).toBeNull();
    expect(reset.error).toBeNull();
    expect(seeded.isIdle).toBe(true);
    expect(seeded.value).toEqual([]);
    expect(seeded.error).toBeNull();
    expect(failed.isFailed).toBe(true);
  });

  it("can discard stale content while loading or failing", () => {
    const ready = AsyncState.ready("previous");

    expect(ready.asLoading(null).value).toBeNull();
    expect(ready.asFailed(new Error("failed"), null).value).toBeNull();
    expect(ready.value).toBe("previous");
  });

  it("preserves a custom failure type through transitions", () => {
    const failed = AsyncState.failed<number, string>("unavailable");

    expect(failed.isFailed).toBe(true);
    expect(failed.error).toBe("unavailable");
    expect(failed.asReady(0).error).toBeNull();
  });
});
