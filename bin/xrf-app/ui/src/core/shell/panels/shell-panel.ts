import { ReactNode } from "react";

/** Which stripe owns a panel. */
export type TShellPanelSide = "left" | "right";

export interface IShellPanel {
  id: string;
  label: string;
  icon: ReactNode;
  /** Left is where a tool browses, right is where it inspects. */
  side?: TShellPanelSide;
  isOpenByDefault?: boolean;
  render: () => ReactNode;
}

/**
 * Selects panels assigned to one side in declaration order.
 *
 * @param panels - Panels to filter.
 * @param side - Side to select.
 * @returns Panels assigned to the selected side.
 */
export function selectPanelsOnSide(panels: ReadonlyArray<IShellPanel>, side: TShellPanelSide): Array<IShellPanel> {
  return panels.filter((panel: IShellPanel) => (panel.side ?? "right") === side);
}
