import { useInjection } from "@wirestate/react";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { ConstructionIcon } from "@xrf/ui/icons/construction";
import { EmptyState } from "@xrf/ui/layout/empty-state";
import { Button } from "@xrf/ui/material/button";
import { ReactElement, useCallback } from "react";

import { KeybindCommandsService } from "@/core/commands";
import { EditorToolbar } from "@/core/shell/chrome/EditorToolbar";
import { CLOSE_TOOL_KEYBIND_COMMAND } from "@/core/shell/commands";

interface IPlannedToolProps {
  /** What this tool will do once it exists, in the present tense. */
  description: string;
}

/**
 * What a tool on the roadmap shows before it is built: a dead end, so it says how to leave it.
 */
export function PlannedTool({ description }: IPlannedToolProps): ReactElement {
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
