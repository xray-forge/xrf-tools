/** The sections settings are grouped into. */
export enum ESettingsSection {
  GENERAL = "general",
  RENDER = "render",
  STORAGE = "storage",
  IPC = "ipc",
  JOBS = "jobs",
  ABOUT = "about",
}

export const SETTINGS_SECTION_LABELS: Readonly<Record<ESettingsSection, string>> = {
  [ESettingsSection.GENERAL]: "General",
  [ESettingsSection.RENDER]: "Rendering",
  [ESettingsSection.STORAGE]: "Storage",
  [ESettingsSection.IPC]: "IPC",
  [ESettingsSection.JOBS]: "Jobs",
  [ESettingsSection.ABOUT]: "About",
};

/** The order the dialog's rail lists the sections in. */
export const SETTINGS_SECTIONS: ReadonlyArray<ESettingsSection> = [
  ESettingsSection.GENERAL,
  ESettingsSection.RENDER,
  ESettingsSection.STORAGE,
  ESettingsSection.IPC,
  ESettingsSection.JOBS,
  ESettingsSection.ABOUT,
];
