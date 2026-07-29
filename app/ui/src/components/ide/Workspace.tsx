import { useStore } from "../../store/useWorkspaceStore";
import { MenuBar } from "./MenuBar";
import { ActivityBar } from "./ActivityBar";
import { PrimarySidebar } from "./PrimarySidebar";
import { EditorArea } from "./EditorArea";
import { RightDock } from "./RightDock";
import { BottomDock } from "./BottomDock";
import { StatusBar } from "./StatusBar";
import { CommandPalette } from "./CommandPalette";

/** The ECAD-native IDE: Cursor/VSCode shell (menu bar, activity bar, docks, palette, status bar)
 *  with EasyEDA-style organization inside (design tree, layers, board canvas, DRC/Log). */
export function Workspace() {
  const { showSidebar, showRightDock, showBottom } = useStore();
  return (
    <div className="ide">
      <MenuBar />
      <div className="ide-body">
        <ActivityBar />
        {showSidebar && <PrimarySidebar />}
        <div className="ide-workarea">
          <div className="ide-panels">
            <EditorArea />
            {showRightDock && <RightDock />}
          </div>
          {showBottom && <BottomDock />}
        </div>
      </div>
      <StatusBar />
      <CommandPalette />
    </div>
  );
}
