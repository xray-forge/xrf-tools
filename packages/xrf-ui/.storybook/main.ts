import * as path from "path";

import type { StorybookConfig } from "@storybook/react-vite";
import { default as tailwindcss } from "@tailwindcss/vite";
import { default as react } from "@vitejs/plugin-react";
import { mergeConfig, runnerImport } from "vite";

import type * as PreviewHead from "./preview-head";

/** Loaded through Vite's runner, as the application loads its own first-paint stylesheet: it reads the kit's TypeScript. */
const PREVIEW_HEAD_ENTRY: string = path.resolve(import.meta.dirname, "preview-head.ts");

let previewHead: Promise<string> | null = null;

async function loadPreviewHead(): Promise<string> {
  const { module } = await runnerImport<typeof PreviewHead>(PREVIEW_HEAD_ENTRY);

  return module.getPreviewHeadHtml();
}

const config: StorybookConfig = {
  framework: "@storybook/react-vite",
  stories: ["../src/**/*.stories.tsx"],
  addons: ["@storybook/addon-a11y"],
  core: {
    disableTelemetry: true,
    // Also silences the notification that links the release-notes page; nothing else in the menus does.
    disableWhatsNewNotifications: true,
  },
  features: {
    // The sidebar's "Get started" checklist, and the guide page that holds the same checklist in the settings menu.
    sidebarOnboardingChecklist: false,
    menuOnboardingChecklist: false,
  },
  previewHead: async (head: string) => {
    previewHead ??= loadPreviewHead().catch((error: unknown) => {
      previewHead = null;
      throw error;
    });

    return `${head}\n${await previewHead}`;
  },
  viteFinal: (base) => mergeConfig(base, { plugins: [tailwindcss(), react()] }),
};

export default config;
