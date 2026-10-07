import { RenderPassCost } from "@/core/ipc/types/xrf-renderer";

/** One row of a frame's GPU readout: a pass alone, or a run of a family's passes summed under the words they share. */
export interface IRenderPassGroup {
  name: string;
  /** Passes summed into it, one for a pass alone. */
  count: number;
  gpuTime: number;
}

/** Passes in a row sharing their first word that collapse into one, as the reflections', the water's or the bloom's. */
const LEAST_GROUPED: number = 3;

/** The words two pass names start with alike. */
function toSharedWords(first: ReadonlyArray<string>, second: ReadonlyArray<string>): ReadonlyArray<string> {
  const shared: Array<string> = [];

  for (
    let index: number = 0;
    index < Math.min(first.length, second.length) && first[index] === second[index];
    index++
  ) {
    shared.push(first[index]);
  }

  return shared;
}

/**
 * @param passes - What each pass cost, in frame order.
 * @returns The readout's rows: each run of at least three passes in a row sharing their first word summed under the
 *   words they all share, every other pass alone.
 */
export function groupRenderPassCosts(passes: ReadonlyArray<RenderPassCost>): ReadonlyArray<IRenderPassGroup> {
  const groups: Array<IRenderPassGroup> = [];
  let index: number = 0;

  while (index < passes.length) {
    const words: ReadonlyArray<string> = passes[index].name.split(" ");
    let shared: ReadonlyArray<string> = words;
    let end: number = index + 1;

    while (end < passes.length && passes[end].name.split(" ")[0] === words[0]) {
      shared = toSharedWords(shared, passes[end].name.split(" "));
      end++;
    }

    const run: ReadonlyArray<RenderPassCost> = passes.slice(index, end);

    if (run.length >= LEAST_GROUPED) {
      groups.push({
        count: run.length,
        gpuTime: run.reduce((total: number, pass: RenderPassCost) => total + (pass.gpuTime ?? 0), 0),
        name: shared.join(" "),
      });
    } else {
      groups.push(...run.map((pass: RenderPassCost) => ({ count: 1, gpuTime: pass.gpuTime ?? 0, name: pass.name })));
    }

    index = end;
  }

  return groups;
}
