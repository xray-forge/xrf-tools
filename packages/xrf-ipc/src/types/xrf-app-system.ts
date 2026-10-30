// Auto-generated rust bindings. Do not edit it manually.

/** How much of one kind the application keeps. */
export type AppStorageArea = {
  kind: AppStorageKind;
  bytes: number;
  files: number;
};

/** What a file the application keeps of its own is for, by where it lies in its data directory. */
export enum EAppStorageKind {
  /** What the renderer rebuilds when it is gone: the driver's compiled pipelines. */
  RENDERER_CACHE = "rendererCache",
  /** Every other cache rebuilt from nothing. */
  OTHER_CACHES = "otherCaches",
  /** This run's log and the last run's. */
  LOGS = "logs",
  /** The backend's preferences. */
  PREFERENCES = "preferences",
  /** Anything else kept there. */
  OTHER = "other",
}

/** Every `EAppStorageKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type AppStorageKind = `${EAppStorageKind}`;

/**
 * What the application keeps in its own data directory, measured file by file: where that is, and how much of each
 * kind, every kind listed whether it holds anything or not.
 */
export type AppStorageUsage = {
  root: string;
  bytes: number;
  areas: Array<AppStorageArea>;
};

/** What everything below one process costs, folded in a single walk of the table. */
export type DescendantUsage = {
  /** Resident set of every process descended from the root, at any depth. */
  residentMemory: number;
  /** How many such processes there are. Zero on a platform that runs the webview in the host process. */
  processes: number;
};

/** What the process holds in the GPU's own memory, as its driver says, and how much of it the renderer made. */
export type GpuMemoryUsage = {
  /**
   * Bytes the process holds there, DLSS's and the driver's counted with the renderer's, which Task Manager's dedicated
   * GPU memory column shows; none where the driver says nothing.
   */
  used: number | null;
  /** Bytes the system lets the process hold there now; none where the driver says nothing. */
  budget: number | null;
  /** Bytes the renderer's GPU allocator holds allocated; none where the backend reports none. */
  allocated: number | null;
};

/** What the application is running on and with, none of which changes while it runs. */
export type HostInfo = {
  /** Tauri the application was linked against. */
  tauriVersion: string;
  /**
   * Webview actually serving the window, which is the runtime installed on the machine rather than a compiled-in
   * version. Absent where the platform cannot be asked, and on Windows where no WebView2 runtime answered.
   */
  webviewVersion: string | null;
  /** Operating system's short name, such as `Windows` or `Ubuntu`. */
  osName: string | null;
  /** Operating system's own version, as it numbers itself. */
  osVersion: string | null;
  /** Kernel behind it, which on Windows is the build number a compatibility report is quoted by. */
  kernelVersion: string | null;
  /** Architecture the binary is executing on, as opposed to the target triple it was built for. */
  arch: string;
  /** Logical processors, which is what the execution pool's width is drawn from. */
  cpuCount: number;
  /** Physical cores, absent where the platform does not distinguish them. */
  physicalCoreCount: number | null;
  /** Total physical memory of the machine, the figure every usage reading is read against. */
  totalMemory: number;
  /** This process's own identifier, for pairing what is shown here with a task manager. */
  pid: number;
};

/** What the page decided when asked to leave what is open or to close the window, which only the page knows. */
export enum ELeaveDecision {
  /** A leave refused while an operation runs. */
  REFUSED_WHILE_BUSY = "refusedWhileBusy",
  /** A leave or exit ignored while another waits for an answer. */
  IGNORED_WHILE_PENDING = "ignoredWhilePending",
  /** Asked before leaving unsaved edits. */
  ASKED_ABOUT_UNSAVED = "askedAboutUnsaved",
  /** Left, with nothing unsaved or once the person agreed. */
  LEFT = "left",
  /** The window asked to close. */
  EXIT_ASKED = "exitAsked",
  /** Asked before closing while an operation runs. */
  ASKED_ABOUT_RUNNING_WORK = "askedAboutRunningWork",
  /** Closed anyway while an operation ran, as the person chose. */
  EXITED_WHILE_BUSY = "exitedWhileBusy",
  /** Kept open, unsaved edits and all, as the person chose. */
  KEPT_OPEN = "keptOpen",
  /** Kept the window open while an operation runs, as the person chose. */
  KEPT_OPEN_WHILE_BUSY = "keptOpenWhileBusy",
  /** A stay refused while edits were being saved before leaving. */
  STAY_REFUSED_WHILE_SAVING = "stayRefusedWhileSaving",
  /** A discard refused while an operation runs. */
  DISCARD_REFUSED_WHILE_BUSY = "discardRefusedWhileBusy",
  /** Unsaved edits discarded before leaving. */
  DISCARDED = "discarded",
  /** A save before leaving refused, with nothing to save, no leave waiting, or an operation running. */
  SAVE_REFUSED = "saveRefused",
  /** Saving edits before leaving. */
  SAVING = "saving",
  /** Saved, then left. */
  SAVED = "saved",
  /** The save wrote nothing, so the page stayed. */
  SAVE_NOT_WRITTEN = "saveNotWritten",
  /** The save failed, so the page stayed. */
  SAVE_FAILED = "saveFailed",
}

/** Every `ELeaveDecision` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LeaveDecision = `${ELeaveDecision}`;

/** What the machine as a whole is using. */
export type MachineUsage = {
  /** Physical memory in use across every process. */
  usedMemory: number;
  /**
   * Physical memory the machine reports as free for a new allocation, which is not `total - used`: the difference is
   * cache the operating system would hand back under pressure.
   */
  availableMemory: number;
};

/** What the application and its webview hold in memory at one instant, and what the application holds on the GPU. */
export type MemoryUsage = {
  /** The backend process itself. */
  application: ProcessMemory;
  /** Every webview process still running, in the order the environment listed them. */
  webview: Array<WebviewProcessMemory>;
  /** The application's GPU memory, none while no renderer runs a GPU. */
  gpu: GpuMemoryUsage | null;
};

/** What a path is right now, for a form field that has to say what it points at before a command runs. */
export type PathDescription = {
  kind: PathKind;
  /** Entries directly inside a directory. */
  entryCount: number | null;
};

/** What a path turned out to be. */
export enum EPathKind {
  MISSING = "missing",
  FILE = "file",
  DIRECTORY = "directory",
}

/** Every `EPathKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type PathKind = `${EPathKind}`;

/** What one process holds in memory. */
export type ProcessMemory = {
  /** Private commit: memory charged to this process alone, resident or not. */
  committed: number;
  /** Working set: physical memory the process occupies, shared pages included. */
  workingSet: number;
  /**
   * Private working set: resident pages no other process shares, which Task Manager's Memory column shows.
   * Absent before Windows 10 1809.
   */
  privateWorkingSet: number | null;
};

/** What one process holds. */
export type ProcessUsage = {
  /** Physical memory the process actually occupies. */
  residentMemory: number;
  /** Address space it has reserved, which is routinely several times the resident set and is not what it costs. */
  virtualMemory: number;
};

/** One reading of what the application costs, and of how long it has been running. */
export type RuntimeSnapshot = {
  /** Milliseconds since the epoch at which the process began, stamped in `main`. */
  startedAt: number;
  /** Milliseconds it has been running, measured monotonically rather than by subtracting two wall-clock readings. */
  uptime: number;
  /** What the backend process itself holds. */
  process: ProcessUsage;
  /** What the processes below it hold, which on this stack is the webview. */
  descendants: DescendantUsage;
  /** What the whole machine is using, for reading the two above against. */
  machine: MachineUsage;
};

/** What a webview process does, as WebView2 names it (`COREWEBVIEW2_PROCESS_KIND`). */
export enum EWebviewProcessKind {
  /** The browser process, which owns the others. */
  BROWSER = "browser",
  /** Runs the page and its workers: the script heaps and every array buffer. */
  RENDERER = "renderer",
  /** Runs the GPU driver on the page's behalf: its compositing and canvases. */
  GPU = "gpu",
  /** A service process, such as the network or audio service. */
  UTILITY = "utility",
  /** Helps the sandbox start the others. */
  SANDBOX_HELPER = "sandboxHelper",
  /** Hosts a plugin, which nothing this application loads uses. */
  PPAPI_PLUGIN = "ppapiPlugin",
  /** Brokers a plugin, which nothing this application loads uses. */
  PPAPI_BROKER = "ppapiBroker",
  /** A kind newer than this build knows. */
  OTHER = "other",
}

/** Every `EWebviewProcessKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type WebviewProcessKind = `${EWebviewProcessKind}`;

/** What one webview process holds. */
export type WebviewProcessMemory = {
  kind: WebviewProcessKind;
  /** The process's own identifier, for pairing it with a task manager. */
  pid: number;
  memory: ProcessMemory;
};
