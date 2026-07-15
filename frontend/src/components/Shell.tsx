import type { ReactNode } from "react";

import { HarnessStatusStrip } from "./HarnessStatusStrip";
import { Sidebar } from "./Sidebar";

interface ShellProps {
  children: ReactNode;
  onRefresh: () => void | Promise<void>;
  refreshPending: boolean;
}

export function Shell({ children, onRefresh, refreshPending }: ShellProps) {
  return (
    <div className="app-shell">
      <Sidebar onRefresh={onRefresh} refreshPending={refreshPending} />
      <div className="app-main-col">
        <HarnessStatusStrip />
        <main className="app-main ui-scrollbar">
          <div className="page-shell">{children}</div>
        </main>
      </div>
    </div>
  );
}
