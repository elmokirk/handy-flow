// Thin wrapper around the generated command so components can close the
// floating pad without importing bindings everywhere.
import { commands } from "@/bindings";

export const hideScratchpad = async (): Promise<void> => {
  await commands.hideScratchpad();
};
