import { useState, useEffect, useRef } from 'react';
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
  const [contextMenu, setContextMenu] = useState<{file: FileItem; x: number; y: number} | null>(null);
  const [renaming, setRenaming] = useState<string | null>(null);
  const [newName, setNewName] = useState('');
  const contextMenuRef = useRef<HTMLDivElement>(null);

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

  function getFileIcon(item: FileItem): React.ReactNode {
    if (item.is_dir) {
      return (
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-amber-500">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
      );
    }
    const ext = item.name.split('.').pop()?.toLowerCase();
    const iconClass = 'text-[var(--color-text-muted)]';
    switch (ext) {
      case 'md':
        return <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={iconClass}><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line></svg>;
      case 'json':
        return <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={iconClass}><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="8" y1="13" x2="16" y2="13"></line><line x1="8" y1="17" x2="16" y2="17"></line></svg>;
      case 'js': case 'ts': case 'jsx': case 'tsx':
        return <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={iconClass}><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline></svg>;
      case 'py':
        return <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={iconClass}><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"></path><path d="M8 12a2 2 0 1 0 4 0 2 2 0 1 0-4 0z"></path><path d="M16 16a2 2 0 1 0 0-4 2 2 0 1 0 0 4z"></path></svg>;
      case 'rs':
        return <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={iconClass}><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path></svg>;
      case 'html': case 'css':
        return <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={iconClass}><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline></svg>;
      case 'png': case 'jpg': case 'jpeg': case 'gif':
        return <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={iconClass}><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>;
      default:
        return <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={iconClass}><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>;
    }
  }

  function handleContextMenu(e: React.MouseEvent, file: FileItem) {
    e.preventDefault();
    setContextMenu({ file, x: e.clientX, y: e.clientY });
  }

  function closeContextMenu() {
    setContextMenu(null);
  }

  function copyPath(file: FileItem) {
    navigator.clipboard.writeText(file.path);
    closeContextMenu();
  }

  function startRename(file: FileItem) {
    setRenaming(file.path);
    setNewName(file.name);
    closeContextMenu();
  }

  function confirmRename(file: FileItem) {
    if (newName.trim() && newName !== file.name) {
      // In a real implementation, this would call an API
      console.log(`Rename ${file.name} to ${newName}`);
      // Update local state for demo
      setFiles(files.map(f => 
        f.path === file.path 
          ? { ...f, name: newName, path: file.path.replace(file.name, newName) }
          : f
      ));
    }
    setRenaming(null);
    setNewName('');
  }

  function deleteFile(file: FileItem) {
    if (confirm(`Delete ${file.name}?`)) {
      // In a real implementation, this would call an API
      console.log(`Delete ${file.path}`);
      setFiles(files.filter(f => f.path !== file.path));
    }
    closeContextMenu();
  }

  // Close context menu on click outside
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (contextMenuRef.current && !contextMenuRef.current.contains(e.target as Node)) {
        closeContextMenu();
      }
    }
    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  }, []);

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4 bg-[var(--color-panel)]">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
              </svg>
            </div>
            <div>
              <h2 className="text-xl font-semibold">Files</h2>
              <p className="text-sm text-[var(--color-text-muted)]">Browse project files</p>
            </div>
          </div>
          <div className="flex gap-1 bg-[var(--color-muted)] p-1 rounded-lg">
            <button
              onClick={() => setViewMode('list')}
              className={`px-3 py-1.5 text-sm rounded-md transition-colors ${
                viewMode === 'list' ? 'bg-[#4248f1] text-white' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
              }`}
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="inline mr-1">
                <line x1="8" y1="6" x2="21" y2="6"></line>
                <line x1="8" y1="12" x2="21" y2="12"></line>
                <line x1="8" y1="18" x2="21" y2="18"></line>
                <line x1="3" y1="6" x2="3.01" y2="6"></line>
                <line x1="3" y1="12" x2="3.01" y2="12"></line>
                <line x1="3" y1="18" x2="3.01" y2="18"></line>
              </svg>
              List
            </button>
            <button
              onClick={() => setViewMode('details')}
              className={`px-3 py-1.5 text-sm rounded-md transition-colors ${
                viewMode === 'details' ? 'bg-[#4248f1] text-white' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
              }`}
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="inline mr-1">
                <rect x="3" y="3" width="7" height="7"></rect>
                <rect x="14" y="3" width="7" height="7"></rect>
                <rect x="14" y="14" width="7" height="7"></rect>
                <rect x="3" y="14" width="7" height="7"></rect>
              </svg>
              Grid
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 flex min-h-0">
        <div className="flex-1 flex flex-col min-w-0 border-r border-border">
          <div className="bg-[var(--color-muted)]/30 p-2 border-b border-border flex items-center gap-2">
            <button
              onClick={() => setCurrentPath('/')}
              className="text-sm px-2 py-1 rounded-xl hover:bg-[#4248f1]/10 transition-colors flex items-center gap-1 text-[#4248f1]"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
                <polyline points="9 22 9 12 15 12 15 22"></polyline>
              </svg>
              Home
            </button>
            <button
              onClick={navigateUp}
              className="text-sm px-2 py-1 rounded hover:bg-[var(--color-muted)] transition-colors flex items-center gap-1 disabled:opacity-50"
              disabled={currentPath === '/'}
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="15 18 9 12 15 6"></polyline>
              </svg>
              Up
            </button>
            <span className="text-[var(--color-text-muted)]">/</span>
            <span className="text-sm font-mono text-[var(--color-text)]">{currentPath}</span>
          </div>

          {loading ? (
            <div className="flex-1 flex items-center justify-center">
              <div className="text-[var(--color-text-muted)] flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="animate-spin">
                  <line x1="12" y1="2" x2="12" y2="6"></line>
                  <line x1="12" y1="18" x2="12" y2="22"></line>
                  <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
                  <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                  <line x1="2" y1="12" x2="6" y2="12"></line>
                  <line x1="18" y1="12" x2="22" y2="12"></line>
                  <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
                  <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
                </svg>
                Loading...
              </div>
            </div>
          ) : error ? (
            <div className="flex-1 flex items-center justify-center">
              <span className="text-red-500 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="15" y1="9" x2="9" y2="15"></line><line x1="9" y1="9" x2="15" y2="15"></line></svg>
                {error}
              </span>
            </div>
          ) : files.length === 0 ? (
            <div className="flex-1 flex items-center justify-center">
              <span className="text-[var(--color-text-muted)] flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                Empty directory
              </span>
            </div>
          ) : viewMode === 'list' ? (
            <div className="flex-1 overflow-y-auto">
              <table className="w-full text-sm">
                <thead className="bg-[var(--color-muted)]/30 sticky top-0">
                  <tr className="text-left">
                    <th className="p-3 font-medium text-[var(--color-text)]">Name</th>
                    <th className="p-3 font-medium text-[var(--color-text)] w-24">Size</th>
                    <th className="p-3 font-medium text-[var(--color-text)] w-32">Modified</th>
                  </tr>
                </thead>
                <tbody>
                  {files.map((file) => (
                    <tr
                      key={file.path}
                      className={`border-t border-[var(--color-border)] hover:bg-[var(--color-muted)]/30 cursor-pointer transition-colors ${
                        selectedFile?.path === file.path ? 'bg-[#4248f1]/5' : ''
                      }`}
                      onClick={() => file.is_dir ? navigateToFolder(file.name) : setSelectedFile(file)}
                      onContextMenu={(e) => handleContextMenu(e, file)}
                    >
                      <td className="p-3">
                        {renaming === file.path ? (
                          <input
                            type="text"
                            value={newName}
                            onChange={(e) => setNewName(e.target.value)}
                            onBlur={() => confirmRename(file)}
                            onKeyDown={(e) => e.key === 'Enter' && confirmRename(file)}
                            onClick={(e) => e.stopPropagation()}
                            autoFocus
                            className="px-2 py-1 border rounded bg-[var(--color-background)] text-[var(--color-text)]"
                          />
                        ) : (
                          <span className="flex items-center gap-2">
                            <span>{getFileIcon(file)}</span>
                            <span className="text-[var(--color-text)]">{file.name}</span>
                          </span>
                        )}
                      </td>
                      <td className="p-3 text-[var(--color-text-muted)]">
                        {formatSize(file.size)}
                      </td>
                      <td className="p-3 text-[var(--color-text-muted)]">
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
                  className={`p-4 border border-[var(--color-border)] rounded-xl text-center cursor-pointer hover:border-[#4248f1]/30 hover:shadow-md transition-all ${
                    selectedFile?.path === file.path ? 'bg-[#4248f1]/5 border-[#4248f1]/30' : 'bg-[var(--color-panel)]'
                  }`}
                >
                  <div className="mb-2 flex justify-center">{getFileIcon(file)}</div>
                  <div className="text-sm truncate text-[var(--color-text)]">{file.name}</div>
                  <div className="text-xs text-[var(--color-text-muted)]">{formatSize(file.size)}</div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="w-80 flex flex-col bg-[var(--color-panel)]">
          <div className="p-4 border-b border-[var(--color-border)]">
            <h3 className="font-semibold">Details</h3>
          </div>
          {selectedFile ? (
            <div className="flex-1 flex flex-col min-h-0 p-4">
              <div className="mb-4 flex justify-center">
                <div className="w-16 h-16 bg-[#4248f1]/10 rounded-xl flex items-center justify-center">
                  {getFileIcon(selectedFile)}
                </div>
              </div>
              <h4 className="font-medium text-center truncate mb-4">{selectedFile.name}</h4>
              <div className="space-y-3 text-sm mb-4">
                <div className="flex justify-between py-2 border-b border-[var(--color-border)]">
                  <span className="text-[var(--color-text-muted)]">Path</span>
                  <span className="text-right truncate max-w-[150px] font-mono text-xs">{selectedFile.path}</span>
                </div>
                <div className="flex justify-between py-2 border-b border-[var(--color-border)]">
                  <span className="text-[var(--color-text-muted)]">Size</span>
                  <span>{formatSize(selectedFile.size)}</span>
                </div>
                <div className="flex justify-between py-2 border-b border-[var(--color-border)]">
                  <span className="text-[var(--color-text-muted)]">Modified</span>
                  <span>{formatDate(selectedFile.modified)}</span>
                </div>
                <div className="flex justify-between py-2">
                  <span className="text-[var(--color-text-muted)]">Type</span>
                  <span>{selectedFile.is_dir ? 'Directory' : 'File'}</span>
                </div>
              </div>
              {!selectedFile.is_dir && (
                <div className="flex-1 flex flex-col min-h-0 mt-2">
                  <h4 className="font-medium mb-2 text-sm">Preview</h4>
                  <pre className="flex-1 overflow-auto text-xs bg-[var(--color-muted)]/30 p-3 rounded-lg font-mono">
                    {fileContent || 'Loading...'}
                  </pre>
                </div>
              )}
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center p-4">
              <p className="text-sm text-[var(--color-text-muted)] text-center">
                Select a file to view details
              </p>
            </div>
          )}
        </div>

        {/* Context Menu */}
        {contextMenu && (
          <div
            ref={contextMenuRef}
            className="fixed bg-[var(--color-panel)] border border-[var(--color-border)] rounded-lg shadow-xl py-1 min-w-[160px] z-50"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            <button
              onClick={() => copyPath(contextMenu.file)}
              className="w-full px-4 py-2 text-left text-sm hover:bg-[var(--color-muted)] flex items-center gap-2 text-[var(--color-text)]"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
              Copy Path
            </button>
            <button
              onClick={() => startRename(contextMenu.file)}
              className="w-full px-4 py-2 text-left text-sm hover:bg-[var(--color-muted)] flex items-center gap-2 text-[var(--color-text)]"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>
              Rename
            </button>
            <button
              onClick={() => deleteFile(contextMenu.file)}
              className="w-full px-4 py-2 text-left text-sm hover:bg-[var(--color-muted)] flex items-center gap-2 text-red-500"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1H7a2-2 2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
              Delete
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
