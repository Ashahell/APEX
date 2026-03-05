import { useState, useEffect } from 'react';
import { apiGet } from '../../lib/api';

interface FileItem {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: number;
}

interface FileContent {
  path: string;
  content: string;
  encoding: string;
}

export function Files() {
  const [currentPath, setCurrentPath] = useState('/');
  const [files, setFiles] = useState<FileItem[]>([]);
  const [selectedFile, setSelectedFile] = useState<FileItem | null>(null);
  const [fileContent, setFileContent] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'list' | 'details'>('list');

  useEffect(() => {
    loadFiles(currentPath);
  }, [currentPath]);

  useEffect(() => {
    if (selectedFile && !selectedFile.is_dir) {
      loadFileContent(selectedFile.path);
    }
  }, [selectedFile]);

  async function loadFiles(path: string) {
    setLoading(true);
    setError(null);
    try {
      const response = await apiGet(`/api/v1/files?path=${encodeURIComponent(path)}`);
      if (response.ok) {
        const data = await response.json();
        setFiles(data);
      } else if (response.status === 404) {
        setFiles([]);
      } else {
        setError('Failed to load files');
      }
    } catch (err) {
      setError('Could not connect to file service');
      setFiles(getMockFiles(path));
    } finally {
      setLoading(false);
    }
  }

  async function loadFileContent(path: string) {
    try {
      const response = await apiGet(`/api/v1/files/content?path=${encodeURIComponent(path)}`);
      if (response.ok) {
        const data: FileContent = await response.json();
        setFileContent(data.content);
      } else {
        setFileContent('// Could not load file content');
      }
    } catch (err) {
      setFileContent('// Could not load file content');
    }
  }

  function getMockFiles(path: string): FileItem[] {
    const base = path === '/' ? '' : path;
    return [
      { name: 'projects', path: `${base}/projects`, is_dir: true, size: 0, modified: Date.now() - 86400000 },
      { name: 'documents', path: `${base}/documents`, is_dir: true, size: 0, modified: Date.now() - 172800000 },
      { name: 'readme.md', path: `${base}/readme.md`, is_dir: false, size: 2048, modified: Date.now() - 86400000 },
      { name: 'config.json', path: `${base}/config.json`, is_dir: false, size: 512, modified: Date.now() - 604800000 },
    ];
  }

  function navigateToFolder(name: string) {
    const newPath = currentPath === '/' ? `/${name}` : `${currentPath}/${name}`;
    setCurrentPath(newPath);
    setSelectedFile(null);
    setFileContent('');
  }

  function navigateUp() {
    if (currentPath === '/') return;
    const parts = currentPath.split('/').filter(Boolean);
    parts.pop();
    setCurrentPath(parts.length === 0 ? '/' : `/${parts.join('/')}`);
    setSelectedFile(null);
    setFileContent('');
  }

  function formatSize(bytes: number): string {
    if (!bytes || bytes === 0) return '-';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp).toLocaleDateString();
  }

  function getFileIcon(item: FileItem): string {
    if (item.is_dir) return '📁';
    const ext = item.name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'md': return '📝';
      case 'json': return '📋';
      case 'js': case 'ts': case 'jsx': case 'tsx': return '📜';
      case 'py': return '🐍';
      case 'rs': return '🦀';
      case 'html': case 'css': return '🌐';
      case 'png': case 'jpg': case 'jpeg': case 'gif': return '🖼️';
      default: return '📄';
    }
  }

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold">Files</h2>
            <p className="text-muted-foreground">Browse project files</p>
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => setViewMode('list')}
              className={`px-3 py-1.5 text-sm rounded ${viewMode === 'list' ? 'bg-primary text-primary-foreground' : 'border'}`}
            >
              List
            </button>
            <button
              onClick={() => setViewMode('details')}
              className={`px-3 py-1.5 text-sm rounded ${viewMode === 'details' ? 'bg-primary text-primary-foreground' : 'border'}`}
            >
              Details
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 flex min-h-0">
        <div className="flex-1 flex flex-col min-w-0 border-r">
          <div className="bg-muted p-2 border-b flex items-center gap-2">
            <button
              onClick={() => setCurrentPath('/')}
              className="text-sm hover:underline"
            >
              Home
            </button>
            <button
              onClick={navigateUp}
              className="text-sm hover:underline"
              disabled={currentPath === '/'}
            >
              ⬆️ Up
            </button>
            <span className="text-muted-foreground">/</span>
            <span className="text-sm font-mono">{currentPath}</span>
          </div>

          {loading ? (
            <div className="flex-1 flex items-center justify-center">
              <span className="text-muted-foreground">Loading...</span>
            </div>
          ) : error ? (
            <div className="flex-1 flex items-center justify-center">
              <span className="text-red-500">{error}</span>
            </div>
          ) : files.length === 0 ? (
            <div className="flex-1 flex items-center justify-center">
              <span className="text-muted-foreground">Empty directory</span>
            </div>
          ) : viewMode === 'list' ? (
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
                  {files.map((file) => (
                    <tr
                      key={file.path}
                      className={`border-t hover:bg-muted/50 cursor-pointer ${
                        selectedFile?.path === file.path ? 'bg-muted' : ''
                      }`}
                      onClick={() => file.is_dir ? navigateToFolder(file.name) : setSelectedFile(file)}
                    >
                      <td className="p-2">
                        <span className="flex items-center gap-2">
                          <span>{getFileIcon(file)}</span>
                          {file.name}
                        </span>
                      </td>
                      <td className="p-2 text-muted-foreground">
                        {formatSize(file.size)}
                      </td>
                      <td className="p-2 text-muted-foreground">
                        {formatDate(file.modified)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="flex-1 overflow-y-auto p-4 grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
              {files.map((file) => (
                <div
                  key={file.path}
                  onClick={() => file.is_dir ? navigateToFolder(file.name) : setSelectedFile(file)}
                  className={`p-4 border rounded-lg text-center cursor-pointer hover:bg-muted/50 ${
                    selectedFile?.path === file.path ? 'bg-muted border-primary' : ''
                  }`}
                >
                  <div className="text-4xl mb-2">{getFileIcon(file)}</div>
                  <div className="text-sm truncate">{file.name}</div>
                  <div className="text-xs text-muted-foreground">{formatSize(file.size)}</div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="w-80 flex flex-col">
          <div className="p-4 border-b">
            <h3 className="font-semibold">Details</h3>
          </div>
          {selectedFile ? (
            <div className="flex-1 flex flex-col min-h-0 p-4">
              <div className="mb-4">
                <div className="text-4xl mb-2">{getFileIcon(selectedFile)}</div>
                <h4 className="font-medium truncate">{selectedFile.name}</h4>
              </div>
              <div className="space-y-2 text-sm mb-4">
                <p><span className="text-muted-foreground">Path:</span> {selectedFile.path}</p>
                <p><span className="text-muted-foreground">Size:</span> {formatSize(selectedFile.size)}</p>
                <p><span className="text-muted-foreground">Modified:</span> {formatDate(selectedFile.modified)}</p>
                <p><span className="text-muted-foreground">Type:</span> {selectedFile.is_dir ? 'Directory' : 'File'}</p>
              </div>
              {!selectedFile.is_dir && (
                <div className="flex-1 flex flex-col min-h-0">
                  <h4 className="font-medium mb-2">Preview</h4>
                  <pre className="flex-1 overflow-auto text-xs bg-muted p-2 rounded font-mono">
                    {fileContent || 'Loading...'}
                  </pre>
                </div>
              )}
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center p-4">
              <p className="text-sm text-muted-foreground text-center">
                Select a file to view details
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
