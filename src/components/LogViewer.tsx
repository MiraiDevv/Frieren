import { useEffect, useRef } from "react";
import { ScrollArea } from "./ui/scroll-area";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "./ui/dialog";

interface LogMessage {
    type: "stdout" | "stderr";
    message: string;
}

interface LogViewerProps {
    isOpen: boolean;
    onOpenChange: (open: boolean) => void;
    logs: LogMessage[];
}

export function LogViewer({ isOpen, onOpenChange, logs }: LogViewerProps) {
    const scrollRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollIntoView({ behavior: "smooth" });
        }
    }, [logs, isOpen]);

    return (
        <Dialog open={isOpen} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-4xl h-[80vh] flex flex-col bg-zinc-950 border-zinc-800">
                <DialogHeader>
                    <DialogTitle className="text-zinc-100">Terminal Output</DialogTitle>
                </DialogHeader>
                <div className="flex-1 min-h-0 bg-black/50 rounded-md border border-zinc-800 p-4 font-mono text-sm overflow-hidden">
                    <ScrollArea className="h-full w-full">
                        <div className="space-y-1">
                            {logs.map((log, index) => (
                                <div
                                    key={index}
                                    className={`${log.type === "stderr" ? "text-red-400" : "text-zinc-300"
                                        } break-all whitespace-pre-wrap`}
                                >
                                    <span className="opacity-50 mr-2 select-none">
                                        {log.type === "stderr" ? "ERR" : "OUT"}
                                    </span>
                                    {log.message}
                                </div>
                            ))}
                            <div ref={scrollRef} />
                        </div>
                    </ScrollArea>
                </div>
            </DialogContent>
        </Dialog>
    );
}
