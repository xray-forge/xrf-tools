import { ReactElement, ReactNode } from "react";

import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IEditorPopoverGroupSectionProps extends BaseComponentProps {
  /** Names the toggle heading the section. */
  label: string;
  /** What it does, or why it cannot be turned on. */
  description?: string;
  isOn: boolean;
  isDisabled?: boolean;
  onToggle: () => void;
  /** The toggle's settings, if it has any. */
  children?: ReactNode;
}

/**
 * One toggle of an `EditorPopoverGroup`, with its settings under it, set apart from the one before.
 */
export function EditorPopoverGroupSection({
  "data-testid": dataTestId = "editor-popover-group-section",
  id,
  className,
  label,
  description,
  isOn,
  isDisabled = false,
  onToggle,
  children,
}: IEditorPopoverGroupSectionProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn(
        "flex flex-col gap-2 border-t border-divider pt-2 first-of-type:border-t-0 first-of-type:pt-0",
        className
      )}
    >
      <CheckboxFormRow
        label={label}
        description={description}
        isChecked={isOn}
        isDisabled={isDisabled}
        onChange={onToggle}
      />

      {children}
    </div>
  );
}
