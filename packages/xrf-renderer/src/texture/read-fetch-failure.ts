/**
 * @param status - What a refusal was answered with.
 * @param text - Its body, where the server answers its message as a JSON string.
 * @returns The message, or the status where the body is none.
 */
export function readFetchFailure(status: number, text: string): string {
  try {
    const message: unknown = JSON.parse(text);

    if (typeof message === "string") {
      return message;
    }
  } catch {
    // Not a message, so the status is all there is to say.
  }

  return `The texture's server answered ${status}`;
}
