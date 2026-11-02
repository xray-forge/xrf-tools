import { runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";

import { CommandOperation } from "@/core/operations/lib/command-operation";

/**
 * Gives an operation an answer as a run would have left it, for a test whose setup cannot wait for one.
 *
 * @param operation - Operation to hold the answer.
 * @param answer - What it holds.
 */
export function mockCommandAnswer<T>(operation: CommandOperation<T>, answer: AsyncState<T>): void {
  runInAction(() => {
    operation["answer"] = answer;
  });
}
