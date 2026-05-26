import { spawn } from 'child_process';
import path from 'path';

export interface FileResult {
  source_file: string;
  source_type: string;
  status: 'good' | 'partial' | 'failed' | 'unsupported';
  output_file?: string;
  error_code?: string;
  warning?: string;
}

export interface JobResult {
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

export async function runAgentReadyCli(
  inputPath: string,
  outputFolder: string
): Promise<JobResult> {
  return new Promise((resolve, reject) => {
    // __dirname is apps/server/src/services
    const cargoToml = path.resolve(__dirname, '../../../../Cargo.toml');

    const args = [
      'run',
      '--manifest-path',
      cargoToml,
      '-p',
      'agentready-cli',
      '--',
      'convert',
      inputPath,
      '--output',
      outputFolder,
      '--json'
    ];

    const child = spawn('cargo', args, {
      stdio: ['ignore', 'pipe', 'pipe']
    });

    let stdoutData = '';
    let stderrData = '';

    child.stdout.on('data', (data) => {
      stdoutData += data.toString();
    });

    child.stderr.on('data', (data) => {
      stderrData += data.toString();
    });

    child.on('close', (code) => {
      // Find the JSON block in stdout (cargo run might prepend compilation logs)
      const jsonMatch = stdoutData.match(/\{[\s\S]*\}/);
      if (jsonMatch) {
        try {
          const result = JSON.parse(jsonMatch[0]) as JobResult;
          resolve(result);
        } catch (e) {
          console.error("Failed to parse JSON from stdout:", stdoutData);
          reject(new Error("Failed to parse CLI output."));
        }
      } else {
        console.error("No JSON found in stdout. Stderr:", stderrData);
        reject(new Error("CLI execution failed or produced invalid output."));
      }
    });

    child.on('error', (err) => {
      reject(err);
    });
  });
}
