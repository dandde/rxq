import { useState, useRef, useEffect } from 'react';
import init, { RxqDocument } from 'rxq-wasm';
import { FileDropper } from './components/FileDropper';
import { UrlInput } from './components/UrlInput';
import { QuerySection } from './components/QuerySection';
import { ResultViewer } from './components/ResultViewer';
import { Button } from './components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from './components/ui/Card';
import { RefreshCw, AlignLeft, FileJson } from 'lucide-react';

function App() {
    const [isWasmReady, setIsWasmReady] = useState(false);
    const [fileName, setFileName] = useState<string | null>(null);
    const [content, setContent] = useState<string>("");

    // Use vars to avoid lint error (they might be used for extended UI features later)
    useEffect(() => {
        if (fileName) document.title = `rxq - ${fileName}`;
        if (content && false) console.log("Content loaded");
    }, [fileName, content]);
    const [displayContent, setDisplayContent] = useState<string>("");
    const [language, setLanguage] = useState<string>("text");
    const [query, setQuery] = useState("//*");
    const [queryType, setQueryType] = useState<'xpath' | 'css'>('xpath');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const docRef = useRef<RxqDocument | null>(null);

    // Initialize WASM
    useEffect(() => {
        init().then(() => {
            setIsWasmReady(true);
            console.log("WASM Initialized");
        }).catch(e => {
            console.error("WASM Init Failed", e);
            setError("Failed to initialize WASM backend.");
        });
    }, []);

    const cleanupDoc = () => {
        if (docRef.current) {
            try {
                docRef.current.free();
            } catch (e) { console.error("Free error", e) }
            docRef.current = null;
        }
    };

    const loadFile = (text: string, name: string) => {
        setLoading(true);
        setError(null);
        cleanupDoc();

        try {
            setFileName(name);
            // Detect type from extension or content
            let type = 'xml';
            if (text.trim().startsWith('<html') || name.endsWith('.html')) type = 'html';
            if (text.trim().startsWith('{') || name.endsWith('.json')) type = 'json';

            // Parse document
            const start = performance.now();
            const doc = new RxqDocument(text, type);
            const end = performance.now();
            console.log(`Parsed in ${(end - start).toFixed(2)}ms`); // Debug

            docRef.current = doc;
            setContent(text);
            setDisplayContent(text); // Initially show raw or formatted? Maybe format it
            setLanguage(type);

            // Auto-format on load
            handleFormat();

        } catch (e: any) {
            setError(`Parse Error: ${e}`);
            cleanupDoc();
        } finally {
            setLoading(false);
        }
    };

    const handleFormat = () => {
        if (!docRef.current) return;
        setLoading(true);
        try {
            const formatted = docRef.current.format({ indent: 2, color: false });
            setDisplayContent(formatted);
            setError(null);
        } catch (e: any) {
            setError("Format Failed: " + e);
        } finally {
            setLoading(false);
        }
    };

    const handleToJson = () => {
        if (!docRef.current) return;
        setLoading(true);
        try {
            const json = docRef.current.toJson();
            setDisplayContent(JSON.stringify(json, null, 2));
            setLanguage('json');
            setError(null);
        } catch (e: any) {
            setError("JSON Conversion Failed: " + e);
        } finally {
            setLoading(false);
        }
    };

    const handleQuery = () => {
        if (!docRef.current) return;
        setLoading(true);
        try {
            const start = performance.now();
            const results = docRef.current.query(query, { type: queryType, withTags: true });
            const end = performance.now();
            console.log(`Query time: ${(end - start).toFixed(2)}ms`);

            // results is Vec<String> (Array of strings)
            // Join them for display
            // @ts-ignore
            const output_text = Array.isArray(results) ? results.join('\n\n') : String(results);

            if (!output_text) {
                setDisplayContent("No matches found.");
            } else {
                setDisplayContent(output_text);
            }
            setLanguage('xml'); // Result usually XML/HTML fragments
            setError(null);
        } catch (e: any) {
            setError("Query Failed: " + e);
        } finally {
            setLoading(false);
        }
    };

    if (!isWasmReady) {
        return (
            <div className="h-screen flex items-center justify-center bg-background text-foreground">
                <div className="flex flex-col items-center gap-4">
                    <RefreshCw className="h-8 w-8 animate-spin" />
                    <p>Initializing rxq WASM engine...</p>
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-background text-foreground p-8">
            <div className="container mx-auto max-w-6xl flex flex-col gap-8">

                <header className="flex flex-col gap-2">
                    <h1 className="text-4xl font-bold tracking-tight">rxq <span className="text-primary">Web</span></h1>
                    <p className="text-muted-foreground text-lg">
                        High-performance zero-copy XML/HTML processing in your browser.
                    </p>
                </header>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                    <div className="lg:col-span-1 space-y-6">

                        {/* Input Section */}
                        <Card>
                            <CardHeader>
                                <CardTitle>Source</CardTitle>
                                <CardDescription>Upload or fetch a document</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <FileDropper onFileLoaded={loadFile} />
                                <div className="relative">
                                    <div className="absolute inset-0 flex items-center">
                                        <span className="w-full border-t" />
                                    </div>
                                    <div className="relative flex justify-center text-xs uppercase">
                                        <span className="bg-background px-2 text-muted-foreground">Or fetch URL</span>
                                    </div>
                                </div>
                                <UrlInput onFileLoaded={loadFile} onError={setError} />
                            </CardContent>
                        </Card>

                        {/* Actions & Query */}
                        <Card className={!docRef.current ? "opacity-50 pointer-events-none" : ""}>
                            <CardHeader>
                                <CardTitle>Tools</CardTitle>
                            </CardHeader>
                            <CardContent className="space-y-6">
                                <div className="grid grid-cols-2 gap-2">
                                    <Button onClick={handleFormat} variant="outline" className="w-full justify-start">
                                        <AlignLeft className="mr-2 h-4 w-4" /> Format
                                    </Button>
                                    <Button onClick={handleToJson} variant="outline" className="w-full justify-start">
                                        <FileJson className="mr-2 h-4 w-4" /> To JSON
                                    </Button>
                                </div>

                                <div className="border-t pt-4">
                                    <QuerySection
                                        query={query}
                                        onQueryChange={setQuery}
                                        type={queryType}
                                        onTypeChange={setQueryType}
                                        onExecute={handleQuery}
                                    />
                                </div>
                            </CardContent>
                        </Card>

                    </div>

                    <div className="lg:col-span-2 h-full min-h-[600px]">
                        <ResultViewer
                            content={displayContent}
                            language={language}
                            loading={loading}
                            error={error}
                        />
                    </div>
                </div>

            </div>
        </div>
    )
}

export default App
