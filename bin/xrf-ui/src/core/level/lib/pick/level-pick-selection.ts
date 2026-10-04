import { Nullable } from "@xrf/types";

import { ERenderSelectionTarget, RenderSelection } from "@/core/ipc/types/xrf-renderer";
import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";
import { toRawColor } from "@/core/render/lib/scene/render-color";
import { VIEWPORT_INK } from "@/core/theme/tokens";

/** The selection's colour: the accent carried on the viewport's backdrop, which is dark whatever the scheme. */
const SELECTION_COLOR: [number, number, number] = toRawColor(Number.parseInt(VIEWPORT_INK.accent.slice(1), 16));

/**
 * @param pick - What a click or a panel picked, or null for nothing.
 * @returns What the viewport marks as selected: a spawned object, or the surface the pick names; null for nothing,
 *   and for a clump of trees drawn as its impostor, which names no surface of its own.
 */
export function toLevelPickSelection(pick: Nullable<TLevelPick>): Nullable<RenderSelection> {
  if (!pick) {
    return null;
  }

  if (pick.kind === ELevelPick.SPAWN) {
    return { color: SELECTION_COLOR, target: { kind: ERenderSelectionTarget.SPAWN, object: pick.object.index } };
  }

  return pick.isImpostor
    ? null
    : {
        color: SELECTION_COLOR,
        target: {
          kind: ERenderSelectionTarget.SURFACE,
          mesh: pick.mesh,
          place: pick.place,
          sector: pick.sector,
          shaderId: pick.shaderId,
        },
      };
}
