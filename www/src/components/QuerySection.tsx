import { HelpCircle, Search } from 'lucide-react';
import { useState } from 'react';
import { Card, CardContent } from './ui/Card';
import { Input } from './ui/Input';
import { Button } from './ui/Button';
import { Label } from './ui/Label';

interface QuerySectionProps {
    query: string;
    onQueryChange: (q: string) => void;
    type: 'xpath' | 'css';
    onTypeChange: (t: 'xpath' | 'css') => void;
    onExecute: () => void;
    disabled?: boolean;
}

export function QuerySection({
    query,
    onQueryChange,
    type,
    onTypeChange,
    onExecute,
    disabled
}: QuerySectionProps) {
    const [showHelp, setShowHelp] = useState(false);

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            onExecute();
        }
    };

    return (
        <div className="space-y-4">
            <div className="flex items-center justify-between">
                <Label>Query Expression</Label>
                <div className="flex space-x-2">
                    <Button
                        variant={type === 'xpath' ? 'default' : 'outline'}
                        size="sm"
                        onClick={() => onTypeChange('xpath')}
                    >
                        XPath
                    </Button>
                    <Button
                        variant={type === 'css' ? 'default' : 'outline'}
                        size="sm"
                        onClick={() => onTypeChange('css')}
                    >
                        CSS
                    </Button>
                </div>
            </div>

            <div className="flex gap-2">
                <div className="relative flex-1">
                    <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                    <Input
                        placeholder={type === 'xpath' ? "//user[@status='active']" : "div#content .item"}
                        value={query}
                        onChange={(e) => onQueryChange(e.target.value)}
                        onKeyDown={handleKeyDown}
                        className="pl-9 font-mono"
                        disabled={disabled}
                    />
                </div>
                <Button onClick={() => setShowHelp(!showHelp)} variant="ghost" size="icon">
                    <HelpCircle className="h-4 w-4" />
                </Button>
            </div>

            {showHelp && (
                <Card className="bg-muted/50 border-dashed">
                    <CardContent className="pt-4 text-sm space-y-2">
                        <h4 className="font-semibold">Query Examples ({type === 'xpath' ? 'XPath' : 'CSS'})</h4>
                        {type === 'xpath' ? (
                            <ul className="list-disc pl-4 space-y-1 text-muted-foreground">
                                <li><code>//tagname</code> - Select all nodes with tagname</li>
                                <li><code>//tag[@attr='val']</code> - Filter by attribute</li>
                                <li><code>/root/child</code> - Absolute path</li>
                                <li><code>//text()</code> - Select text content</li>
                            </ul>
                        ) : (
                            <ul className="list-disc pl-4 space-y-1 text-muted-foreground">
                                <li><code>tagname</code> - Select by tag</li>
                                <li><code>.classname</code> - Select by class</li>
                                <li><code>#id</code> - Select by ID</li>
                                <li><code>parent &gt; children</code> - Direct children</li>
                                <li><code>[attr="val"]</code> - Attribute match</li>
                            </ul>
                        )}
                        <p className="text-xs text-muted-foreground mt-2">
                            rxq uses <a href="https://github.com/dandde/rxq" className="underline" target="_blank">rxq-core</a> for high-performance zero-copy querying.
                        </p>
                    </CardContent>
                </Card>
            )}

            <Button onClick={onExecute} disabled={disabled || !query.trim()} className="w-full">
                Run Query
            </Button>
        </div>
    );
}
