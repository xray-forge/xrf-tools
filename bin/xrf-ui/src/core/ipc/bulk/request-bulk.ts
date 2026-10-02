import { IBulkCall } from "@/core/ipc/bulk/bulk-call";
import { BULK_ENDPOINT } from "@/core/ipc/bulk/bulk-endpoint";
import { IBulkRequest } from "@/core/ipc/bulk/bulk-request";
import { TransportEndpoint } from "@/core/ipc/types/xrf-app";

/** The transport's route taking many calls at once, a JSON array of `{ route, args }` (`TransportServer::BATCH_PATH`). */
const BULK_BATCH_ROUTE: string = "batch";

/**
 * @param call - A route and its arguments.
 * @returns The request making the call, for the page or another thread to fetch, alone or in a batch.
 */
export async function requestBulk(call: IBulkCall): Promise<IBulkRequest> {
  const { origin, token }: TransportEndpoint = await BULK_ENDPOINT.get();

  return {
    batch: { call: JSON.stringify(call), url: `${origin}/${BULK_BATCH_ROUTE}` },
    body: JSON.stringify(call.args),
    headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
    url: `${origin}/${call.route}`,
  };
}
