import { Link, Loader2 } from 'lucide-react';
import { useState } from 'react';
import { Input } from './ui/Input';
import { Button } from './ui/Button';

interface UrlInputProps {
    onFileLoaded: (content: string, name: string) => void;
    onError: (msg: string) => void;
    disabled?: boolean;
}

function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function UrlInput({ onFileLoaded, onError, disabled }: UrlInputProps) {
    const [url, setUrl] = useState('');
    const [loading, setLoading] = useState(false);
    const [fetchTime, setFetchTime] = useState<number | null>(null);
    const [fetchSize, setFetchSize] = useState<number | null>(null);

    const handleFetch = async () => {
        if (!url) return;

        setLoading(true);
        setFetchTime(null);
        setFetchSize(null);
        const startTime = performance.now();

        try {
            // Try direct fetch first
            try {
                const res = await fetch(url);
                if (!res.ok) throw new Error(`Status ${res.status}`);
                const text = await res.text();

                const endTime = performance.now();
                setFetchTime(endTime - startTime);
                setFetchSize(text.length); // Approximation (chars ~= bytes for ASCII, close enough for UI)

                const name = url.split('/').pop() || 'downloaded-file';
                onFileLoaded(text, name);
                return;
            } catch (directError) {
                console.warn("Direct fetch failed, trying CORS proxy...", directError);
            }

            // Fallback to local rxq-server proxy
            // Uses window.location.origin to support whatever host/port we are on
            const proxyUrl = `${window.location.origin}/api/proxy?url=${encodeURIComponent(url)}`;
            const res = await fetch(proxyUrl);
            if (!res.ok) throw new Error(`Proxy Status ${res.status}`);
            const text = await res.text();

            const endTime = performance.now();
            setFetchTime(endTime - startTime);
            setFetchSize(text.length);

            onError("Note: Loaded via Server Proxy (User-Agent spoofed)");
            const name = url.split('/').pop() || 'downloaded-file';
            onFileLoaded(text, name);

        } catch (e: any) {
            onError(`Failed to load URL: ${e.message}`);
        } finally {
            setLoading(false);
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter') handleFetch();
    };

    return (
        <div className="flex flex-col gap-1">
            <div className="flex gap-2">
                <div className="relative flex-1">
                    <Link className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                    <Input
                        placeholder="https://example.com/data.xml"
                        value={url}
                        onChange={(e) => setUrl(e.target.value)}
                        onKeyDown={handleKeyDown}
                        className="pl-9"
                        disabled={disabled || loading}
                    />
                </div>
                <Button onClick={handleFetch} disabled={disabled || loading || !url}>
                    {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : "Fetch"}
                </Button>
            </div>
            {(fetchTime !== null || fetchSize !== null) && (
                <div className="text-xs text-muted-foreground text-right px-1 flex justify-end gap-3">
                    {fetchSize !== null && <span>Size: {formatSize(fetchSize)}</span>}
                    {fetchTime !== null && <span>Time: {fetchTime.toFixed(0)}ms</span>}
                </div>
            )}
        </div>
    );
}
