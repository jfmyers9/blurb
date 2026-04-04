export interface Command {
  id: string;
  label: string;
  keywords: string[];
  group: string;
  execute: () => void;
}

export function createCommands(callbacks: {
  addBook: () => void;
  switchToLibrary: () => void;
  switchToDiary: () => void;
  toggleViewMode: () => void;
  openKindleSync: () => void;
}): Command[] {
  return [
    {
      id: "add-book",
      label: "Add Book",
      keywords: ["new", "create", "add"],
      group: "Actions",
      execute: callbacks.addBook,
    },
    {
      id: "switch-library",
      label: "Switch to Library",
      keywords: ["view", "books", "library", "home"],
      group: "Actions",
      execute: callbacks.switchToLibrary,
    },
    {
      id: "switch-diary",
      label: "Switch to Diary",
      keywords: ["view", "journal", "diary", "entries"],
      group: "Actions",
      execute: callbacks.switchToDiary,
    },
    {
      id: "toggle-view",
      label: "Toggle Grid/List View",
      keywords: ["grid", "list", "view", "layout", "toggle"],
      group: "Actions",
      execute: callbacks.toggleViewMode,
    },
    {
      id: "kindle-sync",
      label: "Open Kindle Sync",
      keywords: ["kindle", "sync", "import", "device"],
      group: "Actions",
      execute: callbacks.openKindleSync,
    },
  ];
}
