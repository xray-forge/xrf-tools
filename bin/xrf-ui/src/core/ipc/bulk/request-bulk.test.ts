import { describe, expect, it } from "@jest/globals";

import { IBulkRequest } from "@/core/ipc/bulk/bulk-request";
import { requestBulk } from "@/core/ipc/bulk/request-bulk";
import { MOCK_TRANSPORT_ENDPOINT, mockFetch } from "@/fixtures/mocks/bulk.mocks";

describe("requestBulk", () => {
  it("answers the request a call is made by, without making it", async () => {
    const request: IBulkRequest = await requestBulk({
      args: { logicalPath: "textures\\a.dds" },
      route: "assets/read_asset",
    });

    expect(request).toEqual({
      body: JSON.stringify({ logicalPath: "textures\\a.dds" }),
      headers: { Authorization: `Bearer ${MOCK_TRANSPORT_ENDPOINT.token}`, "Content-Type": "application/json" },
      url: `${MOCK_TRANSPORT_ENDPOINT.origin}/assets/read_asset`,
    });
    expect(mockFetch).not.toHaveBeenCalled();
  });
});
