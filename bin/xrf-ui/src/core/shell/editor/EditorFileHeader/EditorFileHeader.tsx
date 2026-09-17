import { default as CloseIcon } from "@mui/icons-material/Close";
import { default as DescriptionOutlinedIcon } from "@mui/icons-material/DescriptionOutlined";
import { Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditorFileHeaderProps extends BaseComponentProps {
  /** What is open, as the engine names it, which is the whole path rather than the last segment of it. */
  name: string;
  /** A short note the name does not carry - a size, a count - beside it. */
  caption?: ReactNode;
  /** Marks what kind of thing is open, for a surface whose subject is not a file on disk. */
  icon?: ReactNode;
  /** Actions belonging to what is open, drawn before the close action. */
  actions?: ReactNode;
  /** Names the close action for a surface whose subject is not called a file. */
  closeLabel?: string;
  closeDescription?: string;
  /** Clears the selection. Absent for a surface that opened one subject and has nothing to return to. */
  onClose?: () => void;
}

/**
 * What is open, above the view showing it, with the way to close it again.
 */
export function EditorFileHeader({
  "data-testid": dataTestId = "editor-file-header",
  id,
  className,
  name,
  caption,
  icon = <DescriptionOutlinedIcon className={"text-text-secondary"} fontSize={"small"} />,
  actions,
  closeLabel = "Close file",
  closeDescription = "Clear the selection and close this file",
  onClose,
}: IEditorFileHeaderProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex min-h-header items-center gap-2 border-b border-divider header-band px-3", className)}
    >
      {icon}

      <Typography className={"min-w-0 grow font-monospace"} noWrap variant={"body2"} title={name}>
        {name}
      </Typography>

      {caption ? (
        <Typography className={"text-text-secondary"} noWrap variant={"caption"}>
          {caption}
        </Typography>
      ) : null}

      {actions}

      {onClose ? (
        <EditorIconAction
          data-testid={`${dataTestId}-close`}
          label={closeLabel}
          description={closeDescription}
          icon={<CloseIcon />}
          onClick={onClose}
        />
      ) : null}
    </div>
  );
}
