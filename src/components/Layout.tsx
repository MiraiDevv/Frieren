import { TitleBar } from "./TitleBar";

interface LayoutProps {
    children: React.ReactNode;
}

export function Layout({ children }: LayoutProps) {
    return (
        <div className="min-h-screen flex flex-col bg-background text-foreground rounded-lg overflow-hidden border border-border/50 shadow-2xl">
            <TitleBar />
            <main className="flex-1 pt-10 overflow-auto">
                {children}
            </main>
        </div>
    );
}
