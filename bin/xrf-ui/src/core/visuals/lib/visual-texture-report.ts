import { assertExhaustive, Nullable, Optional } from "@xrf/types";

import { getLocatedAsset } from "@/core/assets/lib/resolution";
import { SelectedVisualDescription } from "@/core/ipc/types/xrf-app";
import { XrayMaterialBumpInput } from "@/core/ipc/types/xrf-material";
import { ERenderTextureState, RenderTextureReport } from "@/core/ipc/types/xrf-renderer";
import { IVisualBumpStatus } from "@/core/visuals/lib/visual-bump";
import { EVisualTextureState, IVisualTextureStatus, toInitialTextureState } from "@/core/visuals/lib/visual-texture";

/** One texture as the renderer reported it, or still on its way. */
interface IVisualTextureOutcome {
  state: EVisualTextureState;
  reason: Nullable<string>;
}

/** What a texture the renderer has not reported yet comes to. */
const PENDING: IVisualTextureOutcome = { reason: null, state: EVisualTextureState.LOADING };

/**
 * @param reference - A texture reference as a shader or a material names it.
 * @returns The same, as two spellings of one name compare.
 */
function toReferenceKey(reference: string): string {
  return reference.replaceAll("/", "\\").toLowerCase();
}

/**
 * @param reports - What the renderer said of every texture its scene samples.
 * @returns Each reference's outcome, by its compared spelling.
 */
function toOutcomes(reports: Array<RenderTextureReport>): Map<string, IVisualTextureOutcome> {
  return new Map(
    reports.map((report: RenderTextureReport): [string, IVisualTextureOutcome] => {
      const { state } = report;

      switch (state.kind) {
        case ERenderTextureState.LOADING:
          return [toReferenceKey(report.reference), PENDING];

        case ERenderTextureState.LOADED:
          return [
            toReferenceKey(report.reference),
            { reason: null, state: state.isExpanded ? EVisualTextureState.DECODED : EVisualTextureState.APPLIED },
          ];

        case ERenderTextureState.MISSING:
          return [toReferenceKey(report.reference), { reason: null, state: EVisualTextureState.UNRESOLVED }];

        case ERenderTextureState.FAILED:
          return [toReferenceKey(report.reference), { reason: state.reason, state: EVisualTextureState.FAILED }];

        default:
          return assertExhaustive(state);
      }
    })
  );
}

/**
 * @param outcomes - Each reported texture's outcome, by its compared spelling.
 * @param input - One half of a bump pair.
 * @returns What became of it, or loading while the renderer has not said.
 */
function toHalfOutcome(
  outcomes: Map<string, IVisualTextureOutcome>,
  input: XrayMaterialBumpInput
): IVisualTextureOutcome {
  return outcomes.get(toReferenceKey(input.reference)) ?? PENDING;
}

/**
 * What became of each submesh's base texture: what its resolution already says, or what the renderer made of it.
 *
 * @param selected - The open visual.
 * @param reports - What the renderer said of every texture it samples; none before it is asked.
 * @returns Each submesh's status, by its index.
 */
export function toVisualTextureStatuses(
  selected: SelectedVisualDescription,
  reports: Array<RenderTextureReport>
): Map<number, IVisualTextureStatus> {
  const outcomes: Map<string, IVisualTextureOutcome> = toOutcomes(reports);

  return new Map(
    selected.dependencies.textures.map((texture) => {
      const initial: EVisualTextureState = toInitialTextureState(texture.resolution);
      const outcome: IVisualTextureOutcome =
        initial === EVisualTextureState.LOADING
          ? (outcomes.get(toReferenceKey(texture.reference)) ?? PENDING)
          : { reason: null, state: initial };

      return [texture.submeshIndex, { ...outcome, submeshIndex: texture.submeshIndex }];
    })
  );
}

/**
 * What became of each bump pair a submesh's material binds, each half on its own.
 *
 * @param selected - The open visual.
 * @param reports - What the renderer said of every texture it samples; none before it is asked.
 * @returns Each submesh's bump status, by its index, for those whose pair has both halves located.
 */
export function toVisualBumpStatuses(
  selected: SelectedVisualDescription,
  reports: Array<RenderTextureReport>
): Map<number, IVisualBumpStatus> {
  const outcomes: Map<string, IVisualTextureOutcome> = toOutcomes(reports);

  return new Map(
    selected.dependencies.textures.flatMap((texture): Array<[number, IVisualBumpStatus]> => {
      const pair = selected.materials[texture.reference]?.bump;

      if (!pair || !getLocatedAsset(pair.bump.resolution) || !getLocatedAsset(pair.companion.resolution)) {
        return [];
      }

      const bump: IVisualTextureOutcome = toHalfOutcome(outcomes, pair.bump);
      const companion: IVisualTextureOutcome = toHalfOutcome(outcomes, pair.companion);
      const reason: Optional<Nullable<string>> = bump.reason ?? companion.reason;

      return [
        [
          texture.submeshIndex,
          { bump: bump.state, companion: companion.state, reason: reason ?? null, submeshIndex: texture.submeshIndex },
        ],
      ];
    })
  );
}
