import { spawn, ChildProcess } from 'child_process';
import path from 'path';
import fs from 'fs';

export interface FileResult {
  source_file: string;
  source_type: string;
  status: 'good' | 'partial' | 'failed' | 'unsupported';
  output_file?: string;
  error_code?: string;
  warning?: string;
  user_message?: string;
}

export interface JobResult {
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

function findCliBinary(): string {
  const candidates = [
    path.resolve(__dirname, '../../../../target/debug/agentready'),
    path.resolve(__dirname, '../../../../target/release/agentready'),
    path.resolve(__dirname, '../../../../target/debug/agentready-cli'),
    path.resolve(__dirname, '../../../../target/release/agentready-cli'),
    path.resolve(__dirname, '../../../../target/debug/agentready.exe'),
    path.resolve(__dirname, '../../../../target/release/agentready.exe'),
    path.resolve(__dirname, '../../../../target/debug/agentready-cli.exe'),
    path.resolve(__dirname, '../../../../target/release/agentready-cli.exe'),
  ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  throw new Error(
    'AgentReady CLI binary not found. Run `cargo build` in the project root first.\n' +
    'Looked in: ' + candidates.join(', ')
  );
}

export async function runAgentReadyCli(
  inputPath: string,
  outputFolder: string,
  onSpawn?: (child: ChildProcess) => void
): Promise<JobResult> {
  const binary = findCliBinary();

  return new Promise((resolve, reject) => {
    const args = [
      'convert',
      inputPath,
      '--output',
      outputFolder,
      '--json',
    ];

    const child = spawn(binary, args, {
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    // Notify caller immediately so they can store the child process reference
    if (onSpawn) {
      onSpawn(child);
    }

    let stdoutData = '';
    let stderrData = '';

    child.stdout.on('data', (data: Buffer) => {
      stdoutData += data.toString();
    });

    child.stderr.on('data', (data: Buffer) => {
      stderrData += data.toString();
    });

    child.on('close', (code) => {
      const lines = stdoutData.trim().split('\n');
      const jsonLine = lines[lines.length - 1];

      if (jsonLine && jsonLine.startsWith('{')) {
        try {
          const result = JSON.parse(jsonLine) as JobResult;
          resolve(result);
        } catch {
          console.error('Failed to parse JSON from stdout:', jsonLine);
          reject(new Error('Failed to parse CLI output.'));
        }
      } else {
        console.error('No JSON found in stdout. Stderr:', stderrData);
        reject(new Error('CLI execution failed or produced invalid output.'));
      }
    });

    child.on('error', (err) => {
      reject(err);
    });
  });
}
