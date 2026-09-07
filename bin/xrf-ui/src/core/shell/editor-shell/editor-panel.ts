import { ReactNode } from "react";

/** Which stripe owns a panel. */
export type TEditorPanelSide = "left" | "right";

export interface IEditorPanel {
  id: string;
  label: string;
  icon: ReactNode;
  /** Left is where an application browses, right is where it inspects. */
  side?: TEditorPanelSide;
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
export function selectPanelsOnSide(panels: ReadonlyArray<IEditorPanel>, side: TEditorPanelSide): Array<IEditorPanel> {
  return panels.filter((panel: IEditorPanel) => (panel.side ?? "right") === side);
}
