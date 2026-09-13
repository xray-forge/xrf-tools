import { default as CloseIcon } from "@mui/icons-material/Close";
import { default as DescriptionOutlinedIcon } from "@mui/icons-material/DescriptionOutlined";
import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { MONOSPACE } from "@/core/theme";
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
  onClose: () => void;
}

/**
 * What is open, above the view showing it, with the way to close it again.
 *
 * The row every browsing application needs and none of them should draw twice: the toolbar says where the open
 * document lives, and this says which one of them is on screen. Closing belongs here rather than on the toolbar
 * because it ends the selection, not the session - the tree stays open behind it.
 */
export function EditorFileHeader({
  "data-testid": dataTestId = "editor-file-header",
  id,
  className,
  name,
  caption,
  icon = <DescriptionOutlinedIcon fontSize={"small"} sx={{ color: "text.secondary" }} />,
  actions,
  closeLabel = "Close file",
  closeDescription = "Clear the selection and close this file",
  onClose,
}: IEditorFileHeaderProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{
        display: "flex",
        alignItems: "center",
        gap: 1,
        minHeight: 40,
        paddingX: 1.5,
        borderBottom: 1,
        borderColor: "divider",
        backgroundColor: "background.paper",
      }}
    >
      {icon}

      <Typography
        noWrap
        variant={"body2"}
        title={name}
        sx={{ flexGrow: 1, minWidth: 0, fontFamily: MONOSPACE.fontFamily }}
      >
        {name}
      </Typography>

      {caption ? (
        <Typography noWrap variant={"caption"} sx={{ color: "text.secondary" }}>
          {caption}
        </Typography>
      ) : null}

      {actions}

      <EditorIconAction
        data-testid={`${dataTestId}-close`}
        label={closeLabel}
        description={closeDescription}
        icon={<CloseIcon />}
        onClick={onClose}
      />
    </Box>
  );
}
