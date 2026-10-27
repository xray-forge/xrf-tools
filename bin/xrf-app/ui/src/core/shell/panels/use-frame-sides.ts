import { Nullable } from "@xrf/types";

import { TShellContributionTool } from "@/core/shell/contributions/shell-contributions";
import {
  IShellContributed,
  useShellContributions,
  useShellContributionTool,
} from "@/core/shell/contributions/use-shell-contributions";
import { IShellPanel, selectPanelsOnSide, TShellPanelSide } from "@/core/shell/panels/shell-panel";
import { IPanelSelection, usePanelSelection } from "@/core/shell/panels/use-panel-selection";
import { IPanelWidth, usePanelWidth } from "@/core/shell/panels/use-panel-width";

interface ISideSelection extends IPanelSelection {
  toolPanels: Array<IShellPanel>;
  shellPanels: Array<IShellPanel>;
}

/**
 * @param side - Side of the frame.
 * @param panels - Every panel published for the open tool and the shell.
 * @param tool - The tool the choice is remembered for.
 * @returns The side's panels, and which is open over the tool's and the shell's together.
 */
function useSideSelection(
  side: TShellPanelSide,
  panels: IShellContributed<IShellPanel>,
  tool: TShellContributionTool
): ISideSelection {
  const toolPanels: Array<IShellPanel> = selectPanelsOnSide(panels.tool, side);
  const shellPanels: Array<IShellPanel> = selectPanelsOnSide(panels.shell, side);
  const selection: IPanelSelection = usePanelSelection(side, [...toolPanels, ...shellPanels], tool);

  return { ...selection, toolPanels, shellPanels };
}

/**
 * @param side - Side of the frame.
 * @param selection - The side's panels and which is open.
 * @param sizing - The side's slot width.
 * @returns The side as the frame draws it.
 */
function toFrameSide(side: TShellPanelSide, selection: ISideSelection, sizing: IPanelWidth): IFrameSide {
  return {
    side,
    toolPanels: selection.toolPanels,
    shellPanels: selection.shellPanels,
    activePanel: selection.activePanel,
    activePanelId: selection.activePanelId,
    width: sizing.width,
    onOpenPanel: selection.onOpenPanel,
    onTogglePanel: selection.onTogglePanel,
    onResize: sizing.onResize,
  };
}

/** What one side of the frame draws: its stripe's panels, the one open in its slot, and the slot's width. */
export interface IFrameSide {
  side: TShellPanelSide;
  /** The open tool's panels, at the top of the stripe. */
  toolPanels: ReadonlyArray<IShellPanel>;
  /** The shell's own panels, in the stripe's footer group. */
  shellPanels: ReadonlyArray<IShellPanel>;
  activePanel: Nullable<IShellPanel>;
  activePanelId: Nullable<string>;
  /** The slot's width, within the budget the open sides share. */
  width: number;
  onOpenPanel: (id: string) => void;
  onTogglePanel: (id: string) => void;
  onResize: (width: number) => void;
}

/** Both sides of the frame by side. */
export type TFrameSides = Readonly<Record<TShellPanelSide, IFrameSide>>;

/**
 * Both sides of the frame, each built the same way: the panels published for the open tool and the shell, the one open
 * on that side as chosen for the open tool, and the slot's width, which shares one budget with the other side.
 *
 * @returns The left and the right side.
 */
export function useFrameSides(): TFrameSides {
  const panels: IShellContributed<IShellPanel> = useShellContributions("panels");
  const tool: TShellContributionTool = useShellContributionTool();

  const left: ISideSelection = useSideSelection("left", panels, tool);
  const right: ISideSelection = useSideSelection("right", panels, tool);

  const openCount: number = (left.activePanel ? 1 : 0) + (right.activePanel ? 1 : 0);
  const leftSizing: IPanelWidth = usePanelWidth("left", openCount);
  const rightSizing: IPanelWidth = usePanelWidth("right", openCount);

  return { left: toFrameSide("left", left, leftSizing), right: toFrameSide("right", right, rightSizing) };
}
