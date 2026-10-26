import { default as ConstructionIcon } from "@mui/icons-material/Construction";
import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { KeybindCommandsService } from "@/core/commands";
import { CLOSE_TOOL_KEYBIND_COMMAND } from "@/core/shell/commands";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EmptyState } from "@/core/ui/layout/EmptyState";

interface IPlannedApplicationProps {
  /** What this application will do once it exists, in the present tense. */
  description: string;
}

/**
 * What an application on the roadmap shows before it is built: a dead end, so it says how to leave it.
 */
export function PlannedApplication({ description }: IPlannedApplicationProps): ReactElement {
  const commandsService: KeybindCommandsService = useInjection(KeybindCommandsService);

  const onClose = useCallback(() => commandsService.execute(CLOSE_TOOL_KEYBIND_COMMAND), [commandsService]);

  return (
    <EditorLayout toolbar={<EditorToolbar />}>
      <EmptyState
        icon={<ConstructionIcon className={"text-text-secondary opacity-55"} />}
        title={"Not implemented yet"}
        description={description}
        action={
          <Button size={"small"} onClick={onClose}>
            Back to all tools
          </Button>
        }
      />
    </EditorLayout>
  );
}
