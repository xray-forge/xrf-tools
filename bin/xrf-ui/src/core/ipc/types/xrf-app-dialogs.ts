// Auto-generated rust bindings. Do not edit it manually.

import { SessionId } from "@/core/ipc/types/xrf-app-core";
import { DialogProjectMode } from "@/core/ipc/types/xrf-dialog";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** What opening a dialogs project was asked to do. */
export type DialogsOpenRequest = {
  sessionId: SessionId;
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** How much of the project to read. */
  mode: DialogProjectMode;
  /** Scope holding the dialog files, or nothing for all of them. */
  dialogsPrefix: string | null;
  /** Scope holding the string tables, or nothing for all of them. */
  translationsPrefix: string | null;
};

/** Identifies a dialog within one committed project. */
export type DialogsReadRequest = {
  sessionId: SessionId;
  logicalPath: string;
  id: string;
  language: string | null;
};
