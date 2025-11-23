import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { X, Minus, Square, Maximize2 } from "lucide-react";
import { ModeToggle } from "./mode-toggle";

export function TitleBar() {
    const [isMaximized, setIsMaximized] = useState(false);

    useEffect(() => {
        const checkMaximized = async () => {
            const win = getCurrentWindow();
            setIsMaximized(await win.isMaximized());
        };

        checkMaximized();

        const unlisten = getCurrentWindow().listen("tauri://resize", checkMaximized);

        return () => {
            unlisten.then((f) => f());
        };
    }, []);

    const minimize = () => getCurrentWindow().minimize();
    const toggleMaximize = async () => {
        const win = getCurrentWindow();
        const maximized = await win.isMaximized();
        if (maximized) {
            win.unmaximize();
        } else {
            win.maximize();
        }
        setIsMaximized(!maximized);
    };
    const close = () => getCurrentWindow().close();

    return (
        <div
            data-tauri-drag-region
            className="h-10 flex items-center justify-between px-4 select-none bg-background/80 backdrop-blur-md border-b z-50 fixed top-0 left-0 right-0 transition-all duration-300 ease-in-out"
        >
            <div className="flex items-center gap-2 group">
                <button
                    onClick={close}
                    className="w-3 h-3 rounded-full bg-red-500 hover:bg-red-600 flex items-center justify-center transition-colors group-hover:opacity-100"
                >
                    <X className="w-2 h-2 text-red-900 opacity-0 group-hover:opacity-100 transition-opacity" />
                </button>
                <button
                    onClick={minimize}
                    className="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-600 flex items-center justify-center transition-colors group-hover:opacity-100"
                >
                    <Minus className="w-2 h-2 text-yellow-900 opacity-0 group-hover:opacity-100 transition-opacity" />
                </button>
                <button
                    onClick={toggleMaximize}
                    className="w-3 h-3 rounded-full bg-green-500 hover:bg-green-600 flex items-center justify-center transition-colors group-hover:opacity-100"
                >
                    {isMaximized ? (
                        <Maximize2 className="w-2 h-2 text-green-900 opacity-0 group-hover:opacity-100 transition-opacity" />
                    ) : (
                        <Square className="w-2 h-2 text-green-900 opacity-0 group-hover:opacity-100 transition-opacity" />
                    )}
                </button>
            </div>

            <div data-tauri-drag-region className="text-xs font-medium text-muted-foreground flex-1 text-center">
                Frieren Downloader
            </div>

            <div className="flex items-center gap-2">
                <ModeToggle />
            </div>
        </div>
    );
}
