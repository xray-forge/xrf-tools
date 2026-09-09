import { ReactElement, ReactNode } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IEditorViewToggleProps extends BaseComponentProps {
  /** Stable accessible name, independent of whether the toggle is on. */
  label: string;
  /** Explains the current state in the tooltip; defaults to the stable toggle label. */
  description?: string;
  icon: ReactNode;
  isOn: boolean;
  /** Why the toggle is unavailable, shown instead of the label. Absent for a toggle that is always offered. */
  unavailableTitle?: string;
  isDisabled?: boolean;
  onToggle: () => void;
}

/**
 * One on/off view toggle of an editor toolbar.
 *
 * Lives with the toolbar rather than with any one editor, because the vocabulary is what makes two tools feel like
 * one program: a person who has learned that a faded icon means the view option is off has learned it everywhere.
 */
export function EditorViewToggle({
  "data-testid": dataTestId = "editor-view-toggle",
  id,
  className,
  label,
  description = label,
  icon,
  isOn,
  unavailableTitle,
  isDisabled = false,
  onToggle,
}: IEditorViewToggleProps): ReactElement {
  return (
    <EditorIconAction
      data-testid={dataTestId}
      aria-pressed={isOn}
      id={id}
      className={className}
      label={label}
      description={isDisabled && unavailableTitle ? unavailableTitle : description}
      icon={icon}
      isDisabled={isDisabled}
      sx={{ opacity: isOn ? 1 : 0.45 }}
      onClick={onToggle}
    />
  );
}
