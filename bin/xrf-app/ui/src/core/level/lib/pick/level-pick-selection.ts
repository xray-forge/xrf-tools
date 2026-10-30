import { ERenderSelectionTarget, RenderSelection } from "@xrf/ipc/types/xrf-renderer";
import { parseHexColor, toRawColor } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { VIEWPORT_INK } from "@xrf/ui/theme";

import { getLevelObjectKind, getLevelSpawnIndex } from "@/core/level/lib/objects/level-object-kinds";
import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";

/** The selection's colour: the accent carried on the viewport's backdrop, which is dark whatever the scheme. */
const SELECTION_COLOR: [number, number, number] = toRawColor(parseHexColor(VIEWPORT_INK.accent));

/**
 * @param pick - What a click or a panel picked, or null for nothing.
 * @returns What the viewport outlines as selected: a level object drawn as a model, or the surface the pick names;
 *   null for nothing, for an object drawn as nothing, which its helper marks instead, and for a clump of trees drawn as
 *   its impostor, which names no surface of its own.
 */
export function toLevelPickSelection(pick: Nullable<TLevelPick>): Nullable<RenderSelection> {
  if (!pick) {
    return null;
  }

  if (pick.kind === ELevelPick.OBJECT) {
    const index: Nullable<number> = getLevelSpawnIndex(pick.object.id);

    return index !== null && getLevelObjectKind(pick.object.kind).isDrawn
      ? { color: SELECTION_COLOR, target: { kind: ERenderSelectionTarget.SPAWN, object: index } }
      : null;
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
