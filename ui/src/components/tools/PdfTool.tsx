import { useState, useRef, useCallback } from 'react';
import { uploadPdf, listPdfDocuments, deletePdfDocument, PdfDocument } from '../../lib/api';

interface PdfUploaderProps {
  onUploadComplete?: (document: PdfDocument) => void;
}

export function PdfUploader({ onUploadComplete }: PdfUploaderProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    
    const files = Array.from(e.dataTransfer.files);
    const pdfFiles = files.filter(f => f.type === 'application/pdf' || f.name.endsWith('.pdf'));
    
    if (pdfFiles.length === 0) {
      setError('Please drop a PDF file');
      return;
    }

    for (const file of pdfFiles) {
      await uploadFile(file);
    }
  }, []);

  const handleFileSelect = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files) return;

    for (const file of Array.from(files)) {
      await uploadFile(file);
    }

    // Reset input
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  }, []);

  const uploadFile = async (file: File) => {
    setUploading(true);
    setError(null);

    try {
      const result = await uploadPdf(file);
      onUploadComplete?.(result.document);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Upload failed');
    } finally {
      setUploading(false);
    }
  };

  return (
    <div className="space-y-4">
      {/* Drop zone */}
      <div
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={() => fileInputRef.current?.click()}
        className={`
          border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors
          ${isDragging 
            ? 'border-indigo-500 bg-indigo-500/10' 
            : 'border-gray-600 hover:border-gray-500 hover:bg-gray-800/50'
          }
          ${uploading ? 'opacity-50 pointer-events-none' : ''}
        `}
      >
        <input
          ref={fileInputRef}
          type="file"
          accept=".pdf,application/pdf"
          multiple
          onChange={handleFileSelect}
          className="hidden"
        />
        
        <div className="space-y-2">
          <div className="text-4xl">📄</div>
          <div className="text-sm text-gray-300">
            {uploading ? (
              <span className="text-indigo-400">Uploading...</span>
            ) : isDragging ? (
              <span className="text-indigo-400">Drop PDF here</span>
            ) : (
              <>
                <span className="text-gray-300">Drag & drop PDF files here</span>
                <br />
                <span className="text-gray-500">or click to browse</span>
              </>
            )}
          </div>
        </div>
      </div>

      {/* Error message */}
      {error && (
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded text-red-400 text-sm">
          {error}
        </div>
      )}
    </div>
  );
}

// ============ PDF Viewer Component ============

interface PdfViewerProps {
  document: PdfDocument;
  onClose?: () => void;
}

export function PdfViewer({ document, onClose }: PdfViewerProps) {
  const [loading, setLoading] = useState(false);
  const [text, setText] = useState<string | null>(document.extracted_text);
  const [error, setError] = useState<string | null>(null);

  const handleExtractText = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await import('../../lib/api').then(m => m.extractPdfText(document.id));
      setText(result.text);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to extract text');
    } finally {
      setLoading(false);
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="bg-gray-900/80 border border-gray-700/50 rounded-lg overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-700/50">
        <div className="flex items-center gap-3">
          <span className="text-2xl">📄</span>
          <div>
            <div className="text-white font-medium">{document.file_name}</div>
            <div className="text-xs text-gray-500">
              {formatFileSize(document.file_size)}
              {document.page_count && ` • ${document.page_count} pages`}
            </div>
          </div>
        </div>
        {onClose && (
          <button
            onClick={onClose}
            className="text-gray-500 hover:text-white"
          >
            ×
          </button>
        )}
      </div>

      {/* Content */}
      <div className="p-4">
        {error && (
          <div className="mb-4 p-3 bg-red-900/30 border border-red-700/50 rounded text-red-400 text-sm">
            {error}
          </div>
        )}

        {text ? (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h4 className="text-sm font-medium text-gray-300">Extracted Text</h4>
              <button
                onClick={handleExtractText}
                disabled={loading}
                className="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white rounded"
              >
                {loading ? 'Extracting...' : 'Re-extract'}
              </button>
            </div>
            <div className="max-h-96 overflow-y-auto p-3 bg-gray-800/50 rounded border border-gray-700/50">
              <pre className="text-sm text-gray-300 whitespace-pre-wrap font-mono">
                {text}
              </pre>
            </div>
          </div>
        ) : (
          <div className="text-center py-8">
            <p className="text-gray-500 mb-4">No text extracted yet</p>
            <button
              onClick={handleExtractText}
              disabled={loading}
              className="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white rounded-lg transition-colors"
            >
              {loading ? 'Extracting text...' : 'Extract Text'}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

// ============ PDF Analyzer Component ============

interface PdfAnalyzerProps {
  document: PdfDocument;
}

export function PdfAnalyzer({ document }: PdfAnalyzerProps) {
  const [prompt, setPrompt] = useState('Summarize this document');
  const [analyzing, setAnalyzing] = useState(false);
  const [analysis, setAnalysis] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleAnalyze = async () => {
    if (!prompt.trim()) return;
    
    setAnalyzing(true);
    setError(null);
    setAnalysis(null);
    
    try {
      const result = await import('../../lib/api').then(m => m.analyzePdf(document.id, prompt));
      setAnalysis(result.analysis);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Analysis failed');
    } finally {
      setAnalyzing(false);
    }
  };

  const quickPrompts = [
    'Summarize this document',
    'What are the key points?',
    'Extract all tables and data',
    'Find specific information about...',
  ];

  return (
    <div className="bg-gray-900/80 border border-gray-700/50 rounded-lg overflow-hidden">
      <div className="p-4 border-b border-gray-700/50">
        <h3 className="text-sm font-medium text-gray-300">Analyze PDF</h3>
      </div>

      <div className="p-4 space-y-4">
        {error && (
          <div className="p-3 bg-red-900/30 border border-red-700/50 rounded text-red-400 text-sm">
            {error}
          </div>
        )}

        {/* Quick prompts */}
        <div className="flex flex-wrap gap-2">
          {quickPrompts.map((q) => (
            <button
              key={q}
              onClick={() => setPrompt(q)}
              className={`px-2 py-1 text-xs rounded transition-colors ${
                prompt === q
                  ? 'bg-indigo-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              {q}
            </button>
          ))}
        </div>

        {/* Custom prompt */}
        <textarea
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          placeholder="Enter your analysis prompt..."
          className="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded text-white text-sm focus:outline-none focus:border-indigo-500"
          rows={3}
        />

        {/* Analyze button */}
        <button
          onClick={handleAnalyze}
          disabled={analyzing || !prompt.trim()}
          className="w-full px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white rounded-lg transition-colors"
        >
          {analyzing ? 'Analyzing...' : 'Analyze'}
        </button>

        {/* Analysis result */}
        {analysis && (
          <div className="mt-4">
            <h4 className="text-sm font-medium text-gray-300 mb-2">Analysis Result</h4>
            <div className="max-h-96 overflow-y-auto p-3 bg-gray-800/50 rounded border border-gray-700/50">
              <pre className="text-sm text-gray-300 whitespace-pre-wrap">
                {analysis}
              </pre>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ============ PDF List Component ============

interface PdfListProps {
  onSelect?: (document: PdfDocument) => void;
  onDelete?: (id: string) => void;
}

export function PdfList({ onSelect, onDelete }: PdfListProps) {
  const [documents, setDocuments] = useState<PdfDocument[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadDocuments = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const docs = await listPdfDocuments();
      setDocuments(docs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load documents');
    } finally {
      setLoading(false);
    }
  }, []);

  useState(() => {
    loadDocuments();
  });

  const handleDelete = async (id: string) => {
    try {
      await deletePdfDocument(id);
      onDelete?.(id);
      setDocuments(prev => prev.filter(d => d.id !== id));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete');
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString();
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="w-5 h-5 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-900/30 border border-red-700/50 rounded text-red-400 text-sm">
        {error}
      </div>
    );
  }

  if (documents.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500">
        No PDF documents uploaded yet
      </div>
    );
  }

  return (
    <ul className="space-y-2">
      {documents.map((doc) => (
        <li
          key={doc.id}
          className="flex items-center justify-between p-3 bg-gray-800/50 rounded border border-gray-700/50 hover:border-gray-600 transition-colors"
        >
          <button
            onClick={() => onSelect?.(doc)}
            className="flex items-center gap-3 flex-1 text-left"
          >
            <span className="text-xl">📄</span>
            <div>
              <div className="text-sm text-white">{doc.file_name}</div>
              <div className="text-xs text-gray-500">
                {formatFileSize(doc.file_size)} • {formatDate(doc.created_at)}
              </div>
            </div>
          </button>
          <button
            onClick={() => handleDelete(doc.id)}
            className="p-2 text-gray-500 hover:text-red-400 transition-colors"
            title="Delete"
          >
            🗑️
          </button>
        </li>
      ))}
    </ul>
  );
}
