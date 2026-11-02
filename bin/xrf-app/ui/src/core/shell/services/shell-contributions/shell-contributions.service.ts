import { Injectable, OnDeprovision } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { ShellContributions } from "@/core/shell/contributions/shell-contributions";
import { IMenuContribution } from "@/core/shell/menu/lib";
import { IShellPanel } from "@/core/shell/panels/shell-panel";
import { isSameStatusSegment, TShellStatusSegment } from "@/core/shell/status-bar/shell-status-segment";

/** The parts of the shell anything may contribute to, by the values each holds. */
export interface IShellSlotValues {
  panels: IShellPanel;
  status: TShellStatusSegment;
  menus: IMenuContribution;
}

export type TShellSlot = keyof IShellSlotValues;

/** Each part of the shell's contributions by name. */
export type TShellSlots = { readonly [S in TShellSlot]: ShellContributions<IShellSlotValues[S]> };

/**
 * What the shell, each tool and any part inside one contribute to the window's panels, status bar and menus,
 * independently of their React trees.
 */
@Injectable()
export class ShellContributionsService implements TShellSlots {
  /** Panels by tool; descriptors and their renderers are kept as published. */
  public readonly panels: ShellContributions<IShellPanel> = new ShellContributions();

  /** Status segments by tool; an unchanged reading republished keeps the one drawn. */
  public readonly status: ShellContributions<TShellStatusSegment> = new ShellContributions(isSameStatusSegment);

  /** Menu entries by tool, each section drawing them in the order their owners first published. */
  public readonly menus: ShellContributions<IMenuContribution> = new ShellContributions();

  @OnDeprovision()
  public onDeprovision(): void {
    runInAction(() => {
      this.panels.clear();
      this.status.clear();
      this.menus.clear();
    });
  }
}
