// Named commands are shared by the menus, palette and keyboard shortcuts.
export type Command = {
  id: string;
  label: string;
  group: "File" | "Edit" | "Sequence" | "Agent" | "Window" | "Help";
  shortcut?: string;
  reason?: string;
  run: () => void | Promise<void>;
};

export function matchesCommand(command: Command, query: string): boolean {
  return `${command.group} ${command.label}`
    .toLowerCase()
    .includes(query.trim().toLowerCase());
}

export async function executeCommand(
  command: Command,
  fail: (message: string) => void,
) {
  if (command.reason) return;
  try {
    await command.run();
  } catch (error) {
    fail(
      error instanceof Error
        ? error.message
        : "The command could not reach ck.",
    );
  }
}
