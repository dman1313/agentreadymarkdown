import React, { useState, useRef } from 'react';
import { UploadCloud, CheckCircle, AlertTriangle, XCircle, FileText, Download, ChevronRight, FileCode2 } from 'lucide-react';
import ReactMarkdown from 'react-markdown';

interface FileResult {
  source_file: string;
  source_type: string;
  status: 'good' | 'partial' | 'failed' | 'unsupported';
  output_file?: string;
  error_code?: string;
  warning?: string;
}

interface JobResult {
  status: 'completed' | 'partial_success' | 'failed' | 'cancelled' | 'validation_error' | 'system_error';
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
  const [dragActive, setDragActive] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === "dragenter" || e.type === "dragover") {
      setDragActive(true);
    } else if (e.type === "dragleave") {
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
    const interval = setInterval(async () => {
      try {
        const res = await fetch(`${API_BASE}/jobs/${id}`);
        const data = await res.json();
        if (data.status === 'Completed' || data.status === 'Failed') {
          clearInterval(interval);
          setJobResult(data.result);
          setAppState('results');
        }
      } catch (err) {
        console.error("Polling error", err);
      }
    }, 1000);
  };

  const fetchPreview = async (fileId: string) => {
    if (!jobId) return;
    try {
      const res = await fetch(`${API_BASE}/jobs/${jobId}/files/${encodeURIComponent(fileId)}/preview`);
      const text = await res.text();
      setPreviewContent(text);
      setAppState('preview');
    } catch (err) {
      console.error(err);
    }
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
        alert("Your browser does not support the File System Access API. Please use the standard download button.");
      }
    } catch (err: any) {
      if (err.name !== 'AbortError') {
        console.error("Save As error:", err);
        alert("Failed to save the file.");
      }
    }
  };

  const renderBadge = (status: string) => {
    switch(status) {
      case 'good': return <span className="badge badge-success"><CheckCircle size={14} className="mr-1"/> Good</span>;
      case 'partial': return <span className="badge badge-partial"><AlertTriangle size={14} className="mr-1"/> Partial</span>;
      default: return <span className="badge badge-error"><XCircle size={14} className="mr-1"/> Failed</span>;
    }
  };

  return (
    <div className="container">
      <header className="header">
        <h1>AgentReadyMarkdown</h1>
        <p>Prepare your data and documents for AI consumption seamlessly.</p>
      </header>

      {appState === 'upload' && (
        <div className="glass-panel">
          <div 
            className={`upload-zone ${dragActive ? 'drag-active' : ''}`}
            onDragEnter={handleDrag}
            onDragLeave={handleDrag}
            onDragOver={handleDrag}
            onDrop={handleDrop}
            onClick={() => fileInputRef.current?.click()}
          >
            <UploadCloud className="upload-icon" />
            <h2>Upload your files</h2>
            <p>Drag and drop, or click to browse (TXT, MD, CSV, DOCX, PDF)</p>
            <input 
              ref={fileInputRef}
              type="file" 
              multiple 
              style={{ display: 'none' }} 
              onChange={handleChange}
            />
          </div>
        </div>
      )}

      {appState === 'converting' && (
        <div className="glass-panel flex-center" style={{ flexDirection: 'column', padding: '4rem 2rem' }}>
          <div style={{ animation: 'spin 2s linear infinite', marginBottom: '1rem' }}>
            <FileCode2 size={48} color="var(--primary)" />
          </div>
          <h2>Converting to AgentReady Markdown...</h2>
          <p>Please wait while we process your files securely.</p>
        </div>
      )}

      {appState === 'results' && jobResult && (
        <div className="glass-panel">
          <div className="flex-between">
            <h2>Conversion Results</h2>
            <div style={{ display: 'flex', gap: '0.5rem' }}>
              <button className="btn btn-secondary" onClick={handleSaveAs}>
                Save As...
              </button>
              <a href={`${API_BASE}/jobs/${jobId}/download`} className="btn btn-primary" download>
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
                  <td>{renderBadge(file.status)}</td>
                  <td>
                    {file.output_file && (
                      <button className="btn btn-secondary" onClick={() => fetchPreview(file.output_file!)} style={{ padding: '0.4rem 0.8rem', fontSize: '0.875rem' }}>
                        Preview <ChevronRight size={14} />
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          
          <div style={{ marginTop: '2rem', textAlign: 'center' }}>
            <button className="btn btn-secondary" onClick={() => setAppState('upload')}>Convert More Files</button>
          </div>
        </div>
      )}

      {appState === 'preview' && (
        <div className="glass-panel">
          <div className="flex-between" style={{ marginBottom: '1.5rem' }}>
            <h2><FileText size={20} style={{ verticalAlign: 'text-bottom', marginRight: '0.5rem' }}/> File Preview</h2>
            <button className="btn btn-secondary" onClick={() => setAppState('results')}>Back to Results</button>
          </div>
          <div className="preview-container">
            <ReactMarkdown>{previewContent}</ReactMarkdown>
          </div>
        </div>
      )}
    </div>
  );
}
