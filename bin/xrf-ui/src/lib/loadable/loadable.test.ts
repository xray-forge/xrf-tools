import { describe, expect, it } from "@jest/globals";

import { ELoadableStatus, Loadable } from "@/lib/loadable";

describe("Loadable", () => {
  it.each([{ value: null }, { value: [] }, { value: 0 }, { value: false }, { value: "" }])(
    "distinguishes idle from a successful empty value: $value",
    ({ value }) => {
      const idle = Loadable.idle(value);
      const ready = idle.asReady();

      expect(idle.status).toBe(ELoadableStatus.IDLE);
      expect(idle.isIdle).toBe(true);
      expect(idle.isReady).toBe(false);
      expect(ready.status).toBe(ELoadableStatus.READY);
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
    const ready = Loadable.ready(value);
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
    const failed = Loadable.failed(new Error("failed"), [1]);
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
    const ready = Loadable.ready("previous");

    expect(ready.asLoading(null).value).toBeNull();
    expect(ready.asFailed(new Error("failed"), null).value).toBeNull();
    expect(ready.value).toBe("previous");
  });

  it("preserves a custom failure type through transitions", () => {
    const failed = Loadable.failed<number, string>("unavailable");

    expect(failed.isFailed).toBe(true);
    expect(failed.error).toBe("unavailable");
    expect(failed.asReady(0).error).toBeNull();
  });
});
