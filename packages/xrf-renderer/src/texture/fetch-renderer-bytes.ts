import { IRendererFetchRequest } from "#/contract/scene/renderer-fetch-request";
import { readFetchFailure } from "#/texture/read-fetch-failure";
import { RendererFetchBatches } from "#/texture/renderer-fetch-batches";
import { IRendererFetchedBytes } from "#/texture/renderer-fetched-bytes";

/** The thread's batches, which every request that may go together joins. */
const BATCHES: RendererFetchBatches = new RendererFetchBatches();

/**
 * @param request - What to fetch: in a batch where it says it may go in one, alone otherwise.
 * @param signal - Aborts it.
 * @returns Its bytes and their media type; refused with the server's own message where it answered one.
 */
export async function fetchRendererBytes(
  request: IRendererFetchRequest,
  signal: AbortSignal
): Promise<IRendererFetchedBytes> {
  if (request.batch) {
    return BATCHES.fetch(request.batch, request.headers, signal);
  }

  const response: Response = await fetch(request.url, {
    body: request.body,
    headers: request.headers,
    method: "POST",
    signal,
  });

  if (!response.ok) {
    throw new Error(readFetchFailure(response.status, await response.text()));
  }

  return { bytes: await response.arrayBuffer(), type: response.headers.get("content-type") };
}
