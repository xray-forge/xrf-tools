import { default as CloseIcon } from "@mui/icons-material/Close";
import { Box } from "@mui/material";
import { CommandBus } from "@wirestate/core";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useRef, useState } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { TEditorPanelSide } from "@/core/shell/editor-shell";
import { IPanelSideCommand, PANEL_CLOSE_COMMAND } from "@/core/shell/panel/panel-messages";
import { Nullable } from "@/lib/types/general";

/**
 * Closes the panel it is drawn in, and draws nothing anywhere else.
 *
 * The side is read from the slot rather than handed down: a panel renders without props, so every header would
 * otherwise repeat what its own descriptor already declares. Reading it also answers whether there is a panel to
 * close at all, which is what keeps the control off the two surfaces that borrow a panel header without being docked.
 */
export function PanelCloseAction(): ReactElement {
  const commandBus: CommandBus = useInjection(CommandBus);
  const ref = useRef<Nullable<HTMLSpanElement>>(null);

  const [side, setSide] = useState<Nullable<TEditorPanelSide>>(null);

  useEffect(() => {
    const hosted: Nullable<string> = ref.current?.closest("[data-panel-side]")?.getAttribute("data-panel-side") ?? null;

    setSide(hosted as Nullable<TEditorPanelSide>);
  }, []);

  return (
    <Box ref={ref} component={"span"} sx={{ display: "inline-flex" }}>
      {side === null ? null : (
        <EditorIconAction
          data-testid={"panel-close"}
          label={"Close panel"}
          description={"Close this panel"}
          icon={<CloseIcon />}
          onClick={() => commandBus.execute<void, IPanelSideCommand>(PANEL_CLOSE_COMMAND, { side }, { optional: true })}
        />
      )}
    </Box>
  );
}
