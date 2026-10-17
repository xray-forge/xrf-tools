import {
  ERenderTextureGroup,
  ERenderTextureState,
  RenderTextureGroup,
  RenderTextureReport,
  RenderTextureState,
} from "@/core/ipc/types/xrf-renderer";
import { ELevelSurfaceDressing, ILevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";

/** One reference the textures have something to say about, for a viewer reporting what a level is missing. */
export interface ILevelTextureProblem {
  reference: string;
  reason: string;
  /** What of the level samples it: its surfaces, or its grass alone. */
  group: RenderTextureGroup;
}

/**
 * What a level's textures came to, as data.
 */
export interface ILevelTextureReport {
  /** References uploaded, which is what a viewer counts. */
  uploaded: number;
  /** References held unread on purpose until a mode that reads them draws, which are neither uploaded nor problems. */
  deferred: number;
  /** Every reference that could not be answered for properly, in the order they were read. */
  problems: ReadonlyArray<ILevelTextureProblem>;
  /** What became of each reference that has been asked for, keyed by it. */
  dressing: ReadonlyMap<string, ILevelSurfaceDressing>;
}

/** Nothing read yet, which is also what a closed level reports. */
export const EMPTY_LEVEL_TEXTURE_REPORT: ILevelTextureReport = {
  deferred: 0,
  dressing: new Map(),
  problems: [],
  uploaded: 0,
};

/**
 * What became of each texture an entry dresses with.
 *
 * @param references - The textures the entry names, in the order it names them.
 * @param report - What the level's textures came to.
 * @returns One answer per reference, in the same order.
 */
export function listLevelSurfaceDressing(
  references: ReadonlyArray<string>,
  report: ILevelTextureReport = EMPTY_LEVEL_TEXTURE_REPORT
): Array<ILevelSurfaceDressing> {
  return references.map(
    (reference: string) =>
      report.dressing.get(reference) ?? { reason: null, reference, state: ELevelSurfaceDressing.UNREAD, upload: null }
  );
}

/**
 * @param described - What a native viewport said became of each texture its level samples.
 * @returns The same, as the level's panels read it.
 */
export function toLevelTextureReport(described: Array<RenderTextureReport>): ILevelTextureReport {
  const dressing: Map<string, ILevelSurfaceDressing> = new Map(
    described.map(({ reference, state }: RenderTextureReport) => [reference, toLevelSurfaceDressing(reference, state)])
  );
  const groups: Map<string, RenderTextureGroup> = new Map(
    described.map(({ group, reference }: RenderTextureReport) => [reference, group])
  );
  const problems: Array<ILevelTextureProblem> = [];
  let uploaded: number = 0;
  let deferred: number = 0;

  dressing.forEach(({ reason, reference, state }: ILevelSurfaceDressing) => {
    if (reason) {
      problems.push({ group: groups.get(reference) ?? ERenderTextureGroup.SURFACES, reason, reference });
    }

    // A stand-in is uploaded as much as a file is; a load on its way is neither yet, and a deferred one neither at all.
    if (state === ELevelSurfaceDressing.DEFERRED) {
      deferred += 1;
    } else if (state !== ELevelSurfaceDressing.FETCHING) {
      uploaded += 1;
    }
  });

  return { deferred, dressing, problems, uploaded };
}

function toLevelSurfaceDressing(reference: string, state: RenderTextureState): ILevelSurfaceDressing {
  switch (state.kind) {
    case ERenderTextureState.LOADING:
      return { reason: null, reference, state: ELevelSurfaceDressing.FETCHING, upload: null };

    case ERenderTextureState.LOADED: {
      const levels: string = `${state.levels} ${state.levels === 1 ? "level" : "levels"}`;
      const expanded: string = state.isExpanded ? " · expanded" : "";

      return {
        reason: null,
        reference,
        state: ELevelSurfaceDressing.UPLOADED,
        upload: `${state.width}×${state.height} · ${state.layout} · ${levels}${expanded}`,
      };
    }

    case ERenderTextureState.DEFERRED:
      return { reason: null, reference, state: ELevelSurfaceDressing.DEFERRED, upload: null };

    case ERenderTextureState.MISSING:
      return {
        reason: "Nothing in the mounted roots answers to it",
        reference,
        state: ELevelSurfaceDressing.STOOD_IN,
        upload: null,
      };

    default:
      return { reason: state.reason, reference, state: ELevelSurfaceDressing.STOOD_IN, upload: null };
  }
}
