import { ReactElement, ReactNode } from "react";

import { SessionRootResource } from "@/core/session/lib/session-root-resource";
import { ToolLoader } from "@/core/shell/chrome/ToolLoader";

interface ISessionToolProps<T> {
  /** The root the tool holds open, which the backend is asked about as the tool provisions. */
  root: SessionRootResource<T>;
  /** Whether the tool has something open to work on; the root holding a value by default. */
  isOpen?: boolean;
  /** What the tool shows with nothing open: the form that opens something. */
  openForm: ReactNode;
  /** What the tool shows with something open. */
  workspace: ReactNode;
}

/**
 * The entry of a tool that holds a session root: the loader until the backend has said what the tool still holds, so a
 * reloaded page never flashes the open form before restoring what was open, then the open form or the workspace.
 */
export function SessionTool<T>({
  root,
  isOpen = root.value !== null,
  openForm,
  workspace,
}: ISessionToolProps<T>): ReactElement {
  if (!root.isRestored) {
    return <ToolLoader />;
  }

  return <>{isOpen ? workspace : openForm}</>;
}
