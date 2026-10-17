import { AsyncState } from "@xrf/async";
import { Nullable } from "@xrf/types";

/**
 * @param description - How asking the renderer what the resident level draws went.
 * @param subject - What a panel waits for from it, as "what it could not draw".
 * @returns What the panel says in place of that, or null once the renderer has said it.
 */
export function describeLevelDescriptionWait(description: AsyncState<void>, subject: string): Nullable<string> {
  if (description.isReady) {
    return null;
  }

  if (description.isFailed) {
    const reason: string = description.error?.message.replace(/\.$/, "") || "the renderer did not say why";

    return `Could not read ${subject}: ${reason}.`;
  }

  if (description.isLoading) {
    return `Reading ${subject}…`;
  }

  return `Not read yet: the renderer says ${subject} once the level is resident.`;
}
