import { describe, expect, it, jest } from "@jest/globals";

import { BulkEndpoint } from "@/core/ipc/bulk/bulk-endpoint";
import { TransportEndpoint } from "@/core/ipc/types/xrf-app";

const ENDPOINT: TransportEndpoint = { origin: "http://127.0.0.1:5", token: "secret" };

describe("BulkEndpoint", () => {
  it("asks the backend once, however many calls ask it", async () => {
    const ask: jest.Mock<() => Promise<TransportEndpoint>> = jest.fn(async () => ENDPOINT);
    const endpoint: BulkEndpoint = new BulkEndpoint(ask);

    const answers: Array<TransportEndpoint> = await Promise.all([endpoint.get(), endpoint.get()]);

    expect(await endpoint.get()).toBe(ENDPOINT);
    expect(answers).toEqual([ENDPOINT, ENDPOINT]);
    expect(ask).toHaveBeenCalledTimes(1);
  });

  // A backend not ready yet is not a backend that never will be.
  it("forgets an ask that failed, so the next call asks again", async () => {
    const ask: jest.Mock<() => Promise<TransportEndpoint>> = jest
      .fn<() => Promise<TransportEndpoint>>()
      .mockRejectedValueOnce(new Error("not yet"))
      .mockResolvedValue(ENDPOINT);
    const endpoint: BulkEndpoint = new BulkEndpoint(ask);

    await expect(endpoint.get()).rejects.toThrow("not yet");
    expect(await endpoint.get()).toBe(ENDPOINT);
    expect(ask).toHaveBeenCalledTimes(2);
  });
});
