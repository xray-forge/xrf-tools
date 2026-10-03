import { IBulkCall } from "@/core/ipc/bulk/bulk-call";
import { BULK_ENDPOINT } from "@/core/ipc/bulk/bulk-endpoint";
import { IBulkRequest } from "@/core/ipc/bulk/bulk-request";
import { TransportEndpoint } from "@/core/ipc/types/xrf-app";

/**
 * @param call - A route and its arguments.
 * @returns The request making the call, for the page to fetch.
 */
export async function requestBulk(call: IBulkCall): Promise<IBulkRequest> {
  const { origin, token }: TransportEndpoint = await BULK_ENDPOINT.get();

  return {
    body: JSON.stringify(call.args),
    headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
    url: `${origin}/${call.route}`,
  };
}
