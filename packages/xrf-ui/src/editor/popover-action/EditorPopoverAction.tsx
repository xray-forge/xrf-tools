import { Nullable } from "@xrf/types";
import {
  CSSProperties,
  MouseEvent,
  MouseEventHandler,
  ReactElement,
  ReactNode,
  useCallback,
  useEffect,
  useId,
  useState,
} from "react";

import { EditorIconAction } from "#/editor/icon-action";
import { BaseComponentProps } from "#/element-types";
import { Popover } from "#/material";

import { TEditorPopoverKind } from "./editor-popover-kind";

/** The corner cue a trigger of each kind draws: a triangle where a click opens, a dot where it switches. */
const CUE_SX: Readonly<Record<TEditorPopoverKind, CSSProperties>> = {
  popover: {
    width: 0,
    height: 0,
    borderLeft: "4px solid transparent",
    borderBottom: "4px solid currentColor",
  },
  toggle: {
    width: 4,
    height: 4,
    borderRadius: "50%",
    backgroundColor: "currentColor",
  },
};

interface IEditorPopoverActionProps extends BaseComponentProps {
  label: string;
  description: string;
  icon: ReactNode;
  isDisabled?: boolean;
  /** The value behind the popover differs from its default, which is painted the same as a toggle that is on. */
  isActive?: boolean;
  /** Side of the trigger on which the content opens. */
  placement?: "top" | "bottom";
  /** What a click does, which its corner cue says: opens it, or switches it (`onClick`). */
  kind?: TEditorPopoverKind;
  /** What a click on the trigger does instead of opening it; a right click opens it either way. */
  onClick?: MouseEventHandler<HTMLButtonElement>;
  /** What it opens; as a function, handed what closes it, as a list of actions closes once one is taken. */
  children: ReactNode | ((close: () => void) => ReactNode);
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
  isActive = false,
  placement = "bottom",
  kind = "popover",
  onClick,
  children,
}: IEditorPopoverActionProps): ReactElement {
  const [anchor, setAnchor] = useState<Nullable<HTMLButtonElement>>(null);
  const dialogId: string = useId();
  const isOpen: boolean = anchor !== null && !isDisabled;

  const onOpen = useCallback((event: MouseEvent<HTMLButtonElement>) => setAnchor(event.currentTarget), []);

  const onContextMenu = useCallback(
    (event: MouseEvent<HTMLButtonElement>) => {
      event.preventDefault();
      onOpen(event);
    },
    [onOpen]
  );

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
        data-cue={kind}
        aria-expanded={isOpen}
        aria-controls={isOpen ? dialogId : undefined}
        id={id}
        className={className}
        label={label}
        description={description}
        icon={icon}
        isActive={isActive}
        isDisabled={isDisabled}
        // Says there is something behind this one and what a click does, which is a kind rather than a state.
        sx={{
          position: "relative",
          "&::after": {
            content: '""',
            position: "absolute",
            right: 2,
            bottom: 2,
            opacity: 0.7,
            ...CUE_SX[kind],
          },
        }}
        onClick={onClick ?? onOpen}
        onContextMenu={onContextMenu}
      />

      <Popover
        anchorEl={anchor}
        open={isOpen}
        anchorOrigin={{ vertical: placement, horizontal: "center" }}
        transformOrigin={{ vertical: placement === "top" ? "bottom" : "top", horizontal: "center" }}
        slotProps={{ paper: { id: dialogId, role: "dialog", "aria-label": label } }}
        onClose={onClose}
      >
        {typeof children === "function" ? children(onClose) : children}
      </Popover>
    </>
  );
}
