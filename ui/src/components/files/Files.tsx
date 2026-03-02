import { useState } from 'react';

interface FileItem {
  name: string;
  type: 'file' | 'folder';
  size?: number;
  modified: string;
}

export function Files() {
  const [currentPath, setCurrentPath] = useState('/');
  const [selectedFile, setSelectedFile] = useState<string | null>(null);

  const mockFiles: FileItem[] = [
    { name: 'projects', type: 'folder', modified: '2026-03-01' },
    { name: 'documents', type: 'folder', modified: '2026-02-28' },
    { name: 'downloads', type: 'folder', modified: '2026-02-25' },
    { name: 'readme.md', type: 'file', size: 1024, modified: '2026-03-01' },
    { name: 'config.json', type: 'file', size: 512, modified: '2026-02-20' },
  ];

  const navigateTo = (path: string) => {
    setCurrentPath(path);
    setSelectedFile(null);
  };

  const formatSize = (bytes?: number) => {
    if (!bytes) return '-';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="p-4 h-full flex flex-col">
      <div className="mb-4">
        <h2 className="text-2xl font-semibold">File Browser</h2>
        <p className="text-muted-foreground">Browse local files</p>
      </div>

      <div className="flex gap-4 flex-1 min-h-0">
        <div className="flex-1 border rounded-lg overflow-hidden flex flex-col">
          <div className="bg-muted p-2 border-b flex items-center gap-2">
            <button
              onClick={() => navigateTo('/')}
              className="text-sm hover:underline"
            >
              Home
            </button>
            <span className="text-muted-foreground">/</span>
            <span className="text-sm">{currentPath}</span>
          </div>

          <div className="flex-1 overflow-y-auto">
            <table className="w-full text-sm">
              <thead className="bg-muted/50 sticky top-0">
                <tr className="text-left">
                  <th className="p-2 font-medium">Name</th>
                  <th className="p-2 font-medium w-24">Size</th>
                  <th className="p-2 font-medium w-32">Modified</th>
                </tr>
              </thead>
              <tbody>
                {mockFiles.map((file) => (
                  <tr
                    key={file.name}
                    className={`border-t hover:bg-muted/50 cursor-pointer ${
                      selectedFile === file.name ? 'bg-muted' : ''
                    }`}
                    onClick={() => setSelectedFile(file.name)}
                  >
                    <td className="p-2">
                      <span className="flex items-center gap-2">
                        <span>{file.type === 'folder' ? '📁' : '📄'}</span>
                        {file.name}
                      </span>
                    </td>
                    <td className="p-2 text-muted-foreground">
                      {formatSize(file.size)}
                    </td>
                    <td className="p-2 text-muted-foreground">{file.modified}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        <div className="w-64 border rounded-lg p-4">
          <h3 className="font-semibold mb-4">Details</h3>
          {selectedFile ? (
            <div className="space-y-2 text-sm">
              <p>
                <span className="text-muted-foreground">File:</span>{' '}
                {selectedFile}
              </p>
              <p>
                <span className="text-muted-foreground">Path:</span>{' '}
                {currentPath}
              </p>
              <button className="w-full mt-4 px-3 py-1.5 text-sm rounded border hover:bg-muted">
                Open in Editor
              </button>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              Select a file to view details
            </p>
          )}
        </div>
      </div>

      <div className="mt-4 p-4 border rounded-lg bg-muted/30">
        <p className="text-sm text-muted-foreground">
          <strong>Note:</strong> File browser is a placeholder. Full implementation
          would connect to a file service API or use WebDAV.
        </p>
      </div>
    </div>
  );
}
