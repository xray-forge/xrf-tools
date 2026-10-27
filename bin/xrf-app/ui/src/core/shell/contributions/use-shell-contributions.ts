import { useInjection } from "@wirestate/react";
import { EMPTY_ARRAY } from "@xrf/collections";
import { SHELL_SESSION_TOOL } from "@xrf/ipc/session";
import { DependencyList, useEffect, useId, useMemo } from "react";

import { useCurrentTool } from "@/core/routing/current-tool.context";
import { IMenuContribution } from "@/core/shell/menu/lib";
import { IShellPanel } from "@/core/shell/panels/shell-panel";
import {
  IShellSlotValues,
  ShellContributionsService,
  TShellSlot,
  TShellSlots,
} from "@/core/shell/services/shell-contributions";
import { TShellStatusSegment } from "@/core/shell/status-bar/shell-status-segment";

import { ShellContributions, TShellContributionTool } from "./shell-contributions";

/**
 * @param slot - Part of the shell.
 * @returns That part's contributions.
 */
function useShellSlot<S extends TShellSlot>(slot: S): ShellContributions<IShellSlotValues[S]> {
  const slots: TShellSlots = useInjection(ShellContributionsService);

  return slots[slot];
}

/**
 * @returns The key the open tool contributes under: its id, or the shell's own where no tool is open.
 */
export function useShellContributionTool(): TShellContributionTool {
  return useCurrentTool()?.id ?? SHELL_SESSION_TOOL;
}

/** One part of the shell's contributions, read apart into what the shell and what the open tool contribute. */
export interface IShellContributed<T> {
  /** The shell's own, drawn in every tool and in the launcher. */
  shell: ReadonlyArray<T>;
  /** The open tool's; nothing in the launcher, whose key is the shell's. */
  tool: ReadonlyArray<T>;
}

/**
 * Reads one part of the shell for the open tool.
 *
 * @param slot - Part of the shell.
 * @returns The shell's own contributions to it and the open tool's, apart, for the caller to order.
 */
export function useShellContributions<S extends TShellSlot>(slot: S): IShellContributed<IShellSlotValues[S]> {
  const contributions: ShellContributions<IShellSlotValues[S]> = useShellSlot(slot);
  const tool: TShellContributionTool = useShellContributionTool();

  return {
    shell: contributions.get(SHELL_SESSION_TOOL),
    tool: tool === SHELL_SESSION_TOOL ? EMPTY_ARRAY : contributions.get(tool),
  };
}

/**
 * Publishes values into one part of the shell while the caller is mounted, releasing them as it unmounts.
 *
 * @param slot - Part of the shell.
 * @param tool - The tool the values belong to.
 * @param values - The values, published again whenever the array changes.
 */
export function useShellContribution<S extends TShellSlot>(
  slot: S,
  tool: TShellContributionTool,
  values: ReadonlyArray<IShellSlotValues[S]>
): void {
  const contributions: ShellContributions<IShellSlotValues[S]> = useShellSlot(slot);
  const owner: string = useId();

  useEffect(() => contributions.publish(owner, tool, values), [contributions, owner, tool, values]);

  useEffect(() => () => contributions.release(owner), [contributions, owner]);
}

/**
 * Publishes panels for the open tool while their owner is mounted. Any number of owners may publish at once.
 *
 * @param createPanels - Creates the current render closures.
 * @param dependencies - Values captured by the factory, checked at call sites by ESLint.
 */
export function useShellPanels(createPanels: () => Array<IShellPanel>, dependencies: DependencyList): void {
  const tool: TShellContributionTool = useShellContributionTool();

  // Call sites are checked as dependency-aware hooks by ESLint.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const panels: Array<IShellPanel> = useMemo(createPanels, dependencies);

  useShellContribution("panels", tool, panels);
}

/**
 * Publishes status segments for the open tool while their owner is mounted. Segments that draw alike keep the ones
 * drawn.
 *
 * @param segments - The current segments.
 */
export function useShellStatus(segments: ReadonlyArray<TShellStatusSegment>): void {
  const tool: TShellContributionTool = useShellContributionTool();

  useShellContribution("status", tool, segments);
}

/**
 * Publishes menu entries for the open tool while their owner is mounted. Any number of owners may publish into one
 * section at once.
 *
 * @param createContributions - Creates the current entries, grouped by the section each goes into.
 * @param dependencies - Values captured by the factory, checked at call sites by ESLint.
 */
export function useShellMenus(createContributions: () => Array<IMenuContribution>, dependencies: DependencyList): void {
  const tool: TShellContributionTool = useShellContributionTool();

  // Call sites are checked as dependency-aware hooks by ESLint.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const contributions: Array<IMenuContribution> = useMemo(createContributions, dependencies);

  useShellContribution("menus", tool, contributions);
}
