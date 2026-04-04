import { open, save } from "@tauri-apps/plugin-dialog";

export function useFilePicker() {
  async function pickDirectory(defaultPath?: string): Promise<string | null> {
    const selected = await open({
      directory: true,
      defaultPath: defaultPath || undefined,
      title: "Select Directory",
    });
    return selected;
  }

  async function pickLogFile(defaultPath?: string): Promise<string | null> {
    const selected = await save({
      defaultPath: defaultPath || undefined,
      filters: [{ name: "Log files", extensions: ["log"] }],
      title: "Select Log File",
    });
    return selected;
  }

  return { pickDirectory, pickLogFile };
}
