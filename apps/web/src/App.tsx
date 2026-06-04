import React, { useState, useRef, useEffect } from 'react';
import { UploadCloud, CheckCircle, AlertTriangle, XCircle, FileText, Download, ChevronRight, FileCode2, Copy, Eye, Code } from 'lucide-react';
import ReactMarkdown from 'react-markdown';

interface FileResult {
  source_file: string;
  source_type: string;
  status: 'good' | 'partial' | 'failed' | 'unsupported';
  output_file?: string;
  error_code?: string;
  warning?: string;
  user_message?: string;
}

interface JobResult {
  status: 'success' | 'partial_success' | 'failed' | 'cancelled' | 'validation_error' | 'system_error';
  exit_code: number;
  input: string;
  output_folder: string;
  export_zip: string;
  summary: {
    total_files: number;
    converted: number;
    partial: number;
    failed: number;
    unsupported: number;
  };
  files: FileResult[];
}

const API_BASE = 'http://localhost:3001/api';

export default function App() {
  const [appState, setAppState] = useState<'upload' | 'converting' | 'results' | 'preview'>('upload');
  const [jobId, setJobId] = useState<string | null>(null);
  const [jobResult, setJobResult] = useState<JobResult | null>(null);
  const [previewContent, setPreviewContent] = useState<string>('');
  const [previewFile, setPreviewFile] = useState<string>('');
  const [previewRaw, setPreviewRaw] = useState(false);
  const [dragActive, setDragActive] = useState(false);
  const [copied, setCopied] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setDragActive(true);
    } else if (e.type === 'dragleave') {
      setDragActive(false);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      await uploadFiles(e.dataTransfer.files);
    }
  };

  const handleChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    e.preventDefault();
    if (e.target.files && e.target.files[0]) {
      await uploadFiles(e.target.files);
    }
  };

  const uploadFiles = async (files: FileList) => {
    setAppState('converting');
    const formData = new FormData();
    for (let i = 0; i < files.length; i++) {
      formData.append('file', files[i]);
    }

    try {
      const res = await fetch(`${API_BASE}/jobs`, {
        method: 'POST',
        body: formData,
      });
      const data = await res.json();
      setJobId(data.id);
      pollJob(data.id);
    } catch (err) {
      console.error(err);
      alert('Failed to upload files.');
      setAppState('upload');
    }
  };

  const pollJob = async (id: string) => {
    intervalRef.current = setInterval(async () => {
      try {
        const res = await fetch(`${API_BASE}/jobs/${id}`);
        const data = await res.json();
        if (data.status === 'Completed' || data.status === 'Failed') {
          if (intervalRef.current) clearInterval(intervalRef.current);
          setJobResult(data.result);
          setAppState('results');
        }
      } catch (err) {
        console.error('Polling error', err);
      }
    }, 1000);
  };

  const fetchPreview = async (fileId: string, fileName: string) => {
    if (!jobId) return;
    try {
      const res = await fetch(`${API_BASE}/jobs/${jobId}/files/${encodeURIComponent(fileId)}/preview`);
      const text = await res.text();
      setPreviewContent(text);
      setPreviewFile(fileName);
      setPreviewRaw(false);
      setAppState('preview');
    } catch (err) {
      console.error(err);
    }
  };

  const copyPreview = async () => {
    await navigator.clipboard.writeText(previewContent);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleSaveAs = async () => {
    if (!jobId) return;
    try {
      if ('showSaveFilePicker' in window) {
        // @ts-ignore
        const handle = await window.showSaveFilePicker({
          suggestedName: `AgentReady-Export-${jobId}.zip`,
          types: [{
            description: 'ZIP Archive',
            accept: { 'application/zip': ['.zip'] },
          }],
        });
        const response = await fetch(`${API_BASE}/jobs/${jobId}/download`);
        const blob = await response.blob();
        const writable = await handle.createWritable();
        await writable.write(blob);
        await writable.close();
      } else {
        const a = document.createElement('a');
        a.href = `${API_BASE}/jobs/${jobId}/download`;
        a.download = `AgentReady-Export-${jobId}.zip`;
        a.click();
      }
    } catch (err: any) {
      if (err.name !== 'AbortError') {
        console.error('Save As error:', err);
      }
    }
  };

  const startOver = () => {
    if (!window.confirm('Start over? Your current conversion will be deleted.')) return;
    if (jobId) {
      fetch(`${API_BASE}/jobs/${jobId}`, { method: 'DELETE' }).catch(() => {});
    }
    setJobId(null);
    setJobResult(null);
    setPreviewContent('');
    setPreviewFile('');
    setAppState('upload');
  };

  const renderBadge = (status: string) => {
    switch (status) {
      case 'good': return <span className="badge badge-success" aria-label="Status: good"><CheckCircle size={14} /> Good</span>;
      case 'partial': return <span className="badge badge-partial" aria-label="Status: partial"><AlertTriangle size={14} /> Partial</span>;
      default: return <span className="badge badge-error" aria-label="Status: failed"><XCircle size={14} /> Failed</span>;
    }
  };

  return (
    <div className="container">
      <header className="header">
        <h1>AgentReady</h1>
        <p className="subtitle">by HumanGoodAI</p>
      </header>

      {appState === 'upload' && (
        <div className="glass-panel">
          <p className="tagline">Is your data agent ready?</p>
          <p className="description">
            AgentReady converts your documents and data into clean Markdown that is easy for AI agents to read.
            Upload your files, preview the output, and download an agent-ready package.
          </p>

          <div
            className={`upload-zone ${dragActive ? 'drag-active' : ''}`}
            role="button"
            tabIndex={0}
            aria-label="Upload files"
            onDragEnter={handleDrag}
            onDragLeave={handleDrag}
            onDragOver={handleDrag}
            onDrop={handleDrop}
            onClick={() => fileInputRef.current?.click()}
            onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') fileInputRef.current?.click(); }}
          >
            <UploadCloud className="upload-icon" />
            <h2>Upload your files</h2>
            <p>Drag and drop, or click to browse</p>
            <input
              ref={fileInputRef}
              type="file"
              multiple
              accept=".txt,.md,.csv,.docx,.pdf"
              style={{ display: 'none' }}
              onChange={handleChange}
            />
          </div>

          <div className="upload-info">
            <div className="info-row">
              <strong>Supported file types:</strong> TXT, Markdown, CSV, DOCX, PDF
            </div>
            <div className="info-row">
              <strong>Limits:</strong> Up to 25 files, 50 MB each
            </div>
            <div className="info-row privacy">
              Your files are processed locally and never uploaded to any cloud service.
            </div>
            <div className="info-row">
              <strong>What happens to my files?</strong> Files are processed entirely on your machine. Nothing leaves your computer.
            </div>
            <div className="info-row">
              <strong>Why Markdown?</strong> Markdown is the most widely supported format for AI agents — clean, readable, and token-efficient.
            </div>
          </div>
        </div>
      )}

      {appState === 'converting' && (
        <div className="glass-panel flex-center" style={{ flexDirection: 'column', padding: '4rem 2rem' }}>
          <div style={{ animation: 'spin 2s linear infinite', marginBottom: '1rem' }}>
            <FileCode2 size={48} color="var(--primary)" />
          </div>
          <h2>Converting to AgentReady Markdown...</h2>
          <p>Your files are being processed securely on your machine.</p>
        </div>
      )}

      {appState === 'results' && jobResult && (
        <div className="glass-panel">
          <div className="flex-between">
            <h2>Conversion Results</h2>
            <div style={{ display: 'flex', gap: '0.5rem' }}>
              <button className="btn btn-secondary" onClick={handleSaveAs} aria-label="Save as">
                Save As...
              </button>
              <a href={`${API_BASE}/jobs/${jobId}/download`} className="btn btn-primary" download aria-label="Download export">
                <Download size={18} /> Download
              </a>
            </div>
          </div>

          <table className="results-table">
            <thead>
              <tr>
                <th>File</th>
                <th>Type</th>
                <th>Status</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              {jobResult.files.map((file, i) => (
                <tr key={i}>
                  <td>{file.source_file}</td>
                  <td style={{ textTransform: 'uppercase' }}>{file.source_type}</td>
                  <td>
                    {renderBadge(file.status)}
                    {file.user_message && (
                      <div className="user-message">{file.user_message}</div>
                    )}
                  </td>
                  <td>
                    {file.output_file && (
                      <button className="btn btn-secondary" onClick={() => fetchPreview(file.output_file!, file.source_file)} style={{ padding: '0.4rem 0.8rem', fontSize: '0.875rem' }} aria-label={`Preview ${file.source_file}`}>
                        Preview <ChevronRight size={14} />
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          <div style={{ marginTop: '2rem', textAlign: 'center' }}>
            <button className="btn btn-secondary" onClick={startOver} aria-label="Convert more files">Convert More Files</button>
          </div>
        </div>
      )}

      {appState === 'preview' && jobResult && (
        <div className="preview-layout">
          <div className="preview-sidebar glass-panel">
            <h3>Files</h3>
            <ul className="file-list">
              {jobResult.files.filter(f => f.output_file).map((file, i) => (
                <li
                  key={i}
                  className={`file-list-item ${file.source_file === previewFile ? 'active' : ''}`}
                  role="button"
                  tabIndex={0}
                  aria-label={`Preview ${file.source_file}`}
                  onClick={() => fetchPreview(file.output_file!, file.source_file)}
                  onKeyDown={(e) => { if (e.key === 'Enter') fetchPreview(file.output_file!, file.source_file); }}
                >
                  <FileText size={14} />
                  <span>{file.source_file}</span>
                  {file.status === 'partial' && <AlertTriangle size={12} className="warning-icon" />}
                </li>
              ))}
            </ul>
          </div>

          <div className="preview-main glass-panel">
            <div className="flex-between" style={{ marginBottom: '1rem' }}>
              <h2><FileText size={20} style={{ verticalAlign: 'text-bottom', marginRight: '0.5rem' }} /> {previewFile}</h2>
              <div style={{ display: 'flex', gap: '0.5rem' }}>
                <button
                  className={`btn btn-secondary ${!previewRaw ? 'active' : ''}`}
                  onClick={() => setPreviewRaw(false)}
                  style={{ padding: '0.4rem 0.8rem', fontSize: '0.875rem' }}
                  aria-label="Rendered view"
                >
                  <Eye size={14} /> Rendered
                </button>
                <button
                  className={`btn btn-secondary ${previewRaw ? 'active' : ''}`}
                  onClick={() => setPreviewRaw(true)}
                  style={{ padding: '0.4rem 0.8rem', fontSize: '0.875rem' }}
                  aria-label="Raw view"
                >
                  <Code size={14} /> Raw
                </button>
                <button
                  className="btn btn-secondary"
                  onClick={copyPreview}
                  style={{ padding: '0.4rem 0.8rem', fontSize: '0.875rem' }}
                  aria-label="Copy content"
                >
                  <Copy size={14} /> {copied ? 'Copied!' : 'Copy'}
                </button>
                <button className="btn btn-secondary" onClick={() => setAppState('results')} style={{ padding: '0.4rem 0.8rem', fontSize: '0.875rem' }} aria-label="Back to results">
                  Back
                </button>
              </div>
            </div>
            <div className="preview-container" aria-label="File preview content">
              {previewRaw ? (
                <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>{previewContent}</pre>
              ) : (
                <ReactMarkdown>{previewContent}</ReactMarkdown>
              )}
            </div>
          </div>
        </div>
      )}

      <footer className="footer">
        <p>A HumanGoodAI tool. Privacy-first by design.</p>
      </footer>
    </div>
  );
}
