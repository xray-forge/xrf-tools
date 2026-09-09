import { Popover } from "@mui/material";
import { MouseEvent, ReactElement, ReactNode, useCallback, useEffect, useId, useState } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IEditorPopoverActionProps extends BaseComponentProps {
  label: string;
  description: string;
  icon: ReactNode;
  isDisabled?: boolean;
  /** Highlights a control whose value differs from its default. */
  isHighlighted?: boolean;
  /** Side of the trigger on which the content opens. */
  placement?: "top" | "bottom";
  children: ReactNode;
}

/**
 * Compact editor trigger that owns a named popover dialog and its open state.
 */
export function EditorPopoverAction({
  "data-testid": dataTestId = "editor-popover-action",
  id,
  className,
  label,
  description,
  icon,
  isDisabled = false,
  isHighlighted = false,
  placement = "bottom",
  children,
}: IEditorPopoverActionProps): ReactElement {
  const [anchor, setAnchor] = useState<Nullable<HTMLButtonElement>>(null);
  const dialogId: string = useId();
  const isOpen: boolean = anchor !== null && !isDisabled;

  const onOpen = useCallback((event: MouseEvent<HTMLButtonElement>) => setAnchor(event.currentTarget), []);

  const onClose = useCallback(() => setAnchor(null), []);

  useEffect(() => {
    if (isDisabled) {
      setAnchor(null);
    }
  }, [isDisabled]);

  return (
    <>
      <EditorIconAction
        data-testid={dataTestId}
        aria-haspopup={"dialog"}
        aria-expanded={isOpen}
        aria-controls={isOpen ? dialogId : undefined}
        id={id}
        className={className}
        label={label}
        description={description}
        icon={icon}
        isHighlighted={isHighlighted}
        isDisabled={isDisabled}
        onClick={onOpen}
      />

      <Popover
        anchorEl={anchor}
        open={isOpen}
        anchorOrigin={{ vertical: placement, horizontal: "center" }}
        transformOrigin={{ vertical: placement === "top" ? "bottom" : "top", horizontal: "center" }}
        slotProps={{ paper: { id: dialogId, role: "dialog", "aria-label": label } }}
        onClose={onClose}
      >
        {children}
      </Popover>
    </>
  );
}
