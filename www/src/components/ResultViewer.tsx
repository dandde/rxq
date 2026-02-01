import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { Card, CardContent, CardHeader, CardTitle } from './ui/Card';

interface ResultViewerProps {
    content: string;
    language: string;
    loading?: boolean;
    error?: string | null;
}

export function ResultViewer({ content, language, loading, error }: ResultViewerProps) {
    if (loading) {
        return (
            <Card className="h-full min-h-[400px] flex items-center justify-center">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </Card>
        );
    }

    if (error) {
        return (
            <Card className="h-full min-h-[400px] border-destructive">
                <CardHeader>
                    <CardTitle className="text-destructive">Error</CardTitle>
                </CardHeader>
                <CardContent>
                    <pre className="whitespace-pre-wrap text-destructive-foreground">{error}</pre>
                </CardContent>
            </Card>
        );
    }

    if (!content) {
        return (
            <Card className="h-full min-h-[400px] flex items-center justify-center text-muted-foreground">
                No output to display
            </Card>
        );
    }

    return (
        <Card className="h-full overflow-hidden flex flex-col">
            <CardHeader className="py-3 px-4 border-b bg-muted/50">
                <div className="flex justify-between items-center">
                    <CardTitle className="text-sm font-mono uppercase">{language}</CardTitle>
                    <div className="text-xs text-muted-foreground">
                        {content.length.toLocaleString()} chars
                    </div>
                </div>
            </CardHeader>
            <div className="flex-1 overflow-auto bg-[#1e1e1e]">
                <SyntaxHighlighter
                    language={language}
                    style={vscDarkPlus}
                    customStyle={{ margin: 0, padding: '1rem', background: 'transparent' }}
                    wrapLongLines={true}
                >
                    {content}
                </SyntaxHighlighter>
            </div>
        </Card>
    );
}
