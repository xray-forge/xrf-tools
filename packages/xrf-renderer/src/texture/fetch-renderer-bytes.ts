import { IRendererFetchRequest } from "#/contract/scene/renderer-fetch-request";
import { IRendererFetchedBytes } from "#/texture/renderer-fetched-bytes";

/**
 * @param request - What to fetch.
 * @param signal - Aborts it.
 * @returns Its bytes and their media type; refused with the server's own message where it answered one.
 */
export async function fetchRendererBytes(
  request: IRendererFetchRequest,
  signal: AbortSignal
): Promise<IRendererFetchedBytes> {
  const response: Response = await fetch(request.url, {
    body: request.body,
    headers: request.headers,
    method: "POST",
    signal,
  });

  if (!response.ok) {
    throw new Error(await readFailure(response));
  }

  return { bytes: await response.arrayBuffer(), type: response.headers.get("content-type") };
}

/**
 * @param response - A refusal.
 * @returns Its message, where the body is the JSON string a server answers with, or its status.
 */
async function readFailure(response: Response): Promise<string> {
  try {
    const message: unknown = JSON.parse(await response.text());

    if (typeof message === "string") {
      return message;
    }
  } catch {
    // Not a message, so the status is all there is to say.
  }

  return `The texture's server answered ${response.status}`;
}
