import type { Args } from "@storybook/react-vite";
import { Optional } from "@xrf/types";
import { useCallback } from "react";
import { useArgs } from "storybook/preview-api";

/**
 * A controlled component's change handler for a story: it writes the change back into the story's args, so the
 * component and the Controls panel show it, then calls the story's own action so the Actions panel logs it.
 *
 * Call it only from a story function: a render component with React state of its own re-renders outside Storybook's
 * hook context, where `useArgs` throws. Keep such state in the args.
 *
 * @param action - The story's handler from its args, usually `fn()`; absent when a story takes the handler away.
 * @param toArgs - The args the new value stands for, given the current args last.
 */
export function useArgChange<TArgs extends Args, TValues extends Array<unknown>>(
  action: Optional<(...values: TValues) => void>,
  toArgs: (...params: [...TValues, NoInfer<TArgs>]) => Partial<TArgs>
): (...values: TValues) => void {
  const [args, updateArgs] = useArgs<TArgs>();

  return useCallback(
    (...values: TValues) => {
      updateArgs(toArgs(...values, args));
      action?.(...values);
    },
    [action, args, toArgs, updateArgs]
  );
}
