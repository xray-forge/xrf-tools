import { beforeEach, describe, expect, it } from "@jest/globals";
import { Optional } from "@xrf/types";

import { fetchBulk } from "#/bulk/fetch-bulk";
import { EAssetsBulkRoute } from "#/commands/assets-bulk";
import { IIpcCommandMetrics, IPC_METRICS } from "#/metrics/index";
import { listMockBulkCalls, MOCK_TRANSPORT_ENDPOINT, mockFetch, mockIpcBulkRoutes } from "#/testing/index";

function recorded(): Optional<IIpcCommandMetrics> {
  return IPC_METRICS.read().commands.find((it: IIpcCommandMetrics) => it.command === "assets|read_asset");
}

describe("fetchBulk", () => {
  beforeEach(() => {
    IPC_METRICS.reset();
  });

  it("posts the route's arguments as JSON to the endpoint, with the launch's token", async () => {
    mockIpcBulkRoutes({ [EAssetsBulkRoute.READ_ASSET]: new Uint8Array([1, 2, 3]) });

    await fetchBulk({ args: { sessionId: "a" }, route: EAssetsBulkRoute.READ_ASSET });

    expect(mockFetch).toHaveBeenCalledWith(`${MOCK_TRANSPORT_ENDPOINT.origin}/assets/read_asset`, {
      body: JSON.stringify({ sessionId: "a" }),
      headers: { Authorization: `Bearer ${MOCK_TRANSPORT_ENDPOINT.token}`, "Content-Type": "application/json" },
      method: "POST",
    });
    expect(listMockBulkCalls(EAssetsBulkRoute.READ_ASSET)).toEqual([{ sessionId: "a" }]);
  });

  it("answers the bytes the route answered", async () => {
    mockIpcBulkRoutes({ [EAssetsBulkRoute.READ_ASSET]: new Uint8Array([9, 8, 7]) });

    const bytes: ArrayBuffer = await fetchBulk({ args: {}, route: EAssetsBulkRoute.READ_ASSET });

    expect(Array.from(new Uint8Array(bytes))).toEqual([9, 8, 7]);
  });

  // The same message an IPC command rejects with, so a caller's error handling needs no second shape.
  it("rejects with the route's own error for a failed route", async () => {
    mockIpcBulkRoutes({
      [EAssetsBulkRoute.READ_ASSET]: () => {
        throw new Error("The visual session has changed or is closed");
      },
    });

    await expect(fetchBulk({ args: {}, route: EAssetsBulkRoute.READ_ASSET })).rejects.toThrow(
      "The visual session has changed or is closed"
    );
  });

  it("rejects with the status where a refusal carries no message", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 401,
      statusText: "Unauthorized",
      text: async () => "",
    } as unknown as Response);

    await expect(fetchBulk({ args: {}, route: EAssetsBulkRoute.READ_ASSET })).rejects.toThrow(
      "The transport answered 401 Unauthorized"
    );
  });

  it("names the route where the transport did not answer at all", async () => {
    mockFetch.mockRejectedValueOnce(new TypeError("Failed to fetch"));

    await expect(fetchBulk({ args: {}, route: EAssetsBulkRoute.READ_ASSET })).rejects.toThrow(
      "The transport did not answer 'assets/read_asset': Failed to fetch"
    );
  });

  it("counts an answer and its bytes under the name its command had", async () => {
    mockIpcBulkRoutes({ [EAssetsBulkRoute.READ_ASSET]: new Uint8Array(12) });

    await fetchBulk({ args: {}, route: EAssetsBulkRoute.READ_ASSET });

    expect(recorded()).toMatchObject({ calls: 1, failures: 0, received: 12 });
  });

  it("counts a failure apart from the answers", async () => {
    mockIpcBulkRoutes({
      [EAssetsBulkRoute.READ_ASSET]: () => {
        throw new Error("gone");
      },
    });

    await expect(fetchBulk({ args: {}, route: EAssetsBulkRoute.READ_ASSET })).rejects.toThrow("gone");

    expect(recorded()).toMatchObject({ calls: 0, failures: 1 });
    expect(IPC_METRICS.read().inFlight).toBe(0);
  });
});
