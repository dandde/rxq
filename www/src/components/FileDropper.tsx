import React, { useCallback, useState } from 'react';
import { Upload, CheckCircle, AlertCircle } from 'lucide-react';
import { cn } from '../lib/utils';
import { Card } from './ui/Card';

interface FileDropperProps {
    onFileLoaded: (content: string, name: string) => void;
    className?: string;
}

export function FileDropper({ onFileLoaded, className }: FileDropperProps) {
    const [isDragging, setIsDragging] = useState(false);
    const [fileName, setFileName] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    const processFile = (file: File) => {
        setError(null);
        setFileName(file.name);

        const reader = new FileReader();
        reader.onload = (e) => {
            const content = e.target?.result as string;
            onFileLoaded(content, file.name);
        };
        reader.onerror = () => {
            setError("Failed to read file");
        };
        reader.readAsText(file);
    };

    const handleDragOver = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(true);
    }, []);

    const handleDragLeave = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(false);
    }, []);

    const handleDrop = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(false);

        const files = e.dataTransfer.files;
        if (files.length > 0) {
            processFile(files[0]);
        }
    }, []);

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files.length > 0) {
            processFile(e.target.files[0]);
        }
    };

    return (
        <Card
            className={cn(
                "relative flex flex-col items-center justify-center p-8 border-2 border-dashed transition-colors cursor-pointer min-h-[200px]",
                isDragging ? "border-primary bg-primary/5" : "border-muted-foreground/25 hover:border-primary/50",
                fileName ? "bg-accent/50" : "",
                className
            )}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            onClick={() => document.getElementById('file-upload')?.click()}
        >
            <input
                id="file-upload"
                type="file"
                className="hidden"
                onChange={handleInputChange}
                accept=".xml,.html,.svg,.json"
            />

            <div className="flex flex-col items-center gap-2 text-center">
                {error ? (
                    <>
                        <div className="p-3 rounded-full bg-destructive/10 text-destructive">
                            <AlertCircle size={32} />
                        </div>
                        <div className="text-sm font-medium text-destructive">{error}</div>
                    </>
                ) : fileName ? (
                    <>
                        <div className="p-3 rounded-full bg-primary/10 text-primary">
                            <CheckCircle size={32} />
                        </div>
                        <div className="text-sm font-medium">{fileName}</div>
                        <p className="text-xs text-muted-foreground">Click or drop to replace</p>
                    </>
                ) : (
                    <>
                        <div className="p-3 rounded-full bg-secondary text-secondary-foreground mb-2">
                            <Upload size={24} />
                        </div>
                        <h3 className="font-semibold text-lg">Drop XML/HTML file here</h3>
                        <p className="text-sm text-muted-foreground">
                            or click to browse
                        </p>
                    </>
                )}
            </div>
        </Card>
    );
}
