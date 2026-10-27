import { AsyncState } from "@xrf/async";
import { Nullable } from "@xrf/types";

import { ILevelResident } from "@/core/level/lib/render/level-resident";

/**
 * @param resident - How asking the renderer what the resident level draws went.
 * @param subject - What a panel waits for from it, as "what it could not draw".
 * @returns What the panel says in place of that, or null once the renderer has said it.
 */
export function describeLevelResidentWait(resident: AsyncState<ILevelResident>, subject: string): Nullable<string> {
  if (resident.isReady) {
    return null;
  }

  if (resident.isFailed) {
    const reason: string = resident.error?.message.replace(/\.$/, "") || "the renderer did not say why";

    return `Could not read ${subject}: ${reason}.`;
  }

  if (resident.isLoading) {
    return `Reading ${subject}…`;
  }

  return `Not read yet: the renderer says ${subject} once the level is resident.`;
}
