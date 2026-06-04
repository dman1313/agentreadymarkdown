import { FastifyInstance, FastifyRequest, FastifyReply } from 'fastify';
import { v4 as uuidv4 } from 'uuid';
import { ChildProcess } from 'child_process';
import fs from 'fs';
import path from 'path';
import os from 'os';
import util from 'util';
import { pipeline } from 'stream';
import { runAgentReadyCli, JobResult } from '../services/cli';

const pump = util.promisify(pipeline);

const ALLOWED_EXTENSIONS = ['.txt', '.md', '.csv', '.docx', '.pdf', '.xlsx'];
const MAX_FILES = 25;

// In-memory job state for V1
interface JobState {
  id: string;
  status: 'Converting' | 'Completed' | 'Failed' | 'Cancelling' | 'Cancelled';
  result: JobResult | null;
  inputFolder: string;
  outputFolder: string;
  childProcess: ChildProcess | null;
}

const jobs = new Map<string, JobState>();

export default async function jobRoutes(fastify: FastifyInstance) {
  // POST /api/jobs - Upload files and start conversion
  fastify.post('/api/jobs', async (request: FastifyRequest, reply: FastifyReply) => {
    const parts = request.files();
    const jobId = `job-${uuidv4()}`;
    const baseDir = path.join(os.tmpdir(), `agentready-${jobId}`);
    const inputDir = path.join(baseDir, 'input');
    const outputDir = path.join(baseDir, 'output');

    fs.mkdirSync(inputDir, { recursive: true });

    let fileCount = 0;
    for await (const part of parts) {
      if (part.type !== 'file') continue;

      // Sanitize filename to prevent path traversal
      const safeName = path.basename(part.filename)
        .replace(/[^a-zA-Z0-9._-]/g, '_');

      // Validate extension
      const ext = path.extname(safeName).toLowerCase();
      if (!ALLOWED_EXTENSIONS.includes(ext)) {
        continue; // skip unsupported files
      }

      // Enforce file count limit
      if (fileCount >= MAX_FILES) {
        continue;
      }

      const savePath = path.join(inputDir, safeName);
      await pump(part.file, fs.createWriteStream(savePath));
      fileCount++;
    }

    if (fileCount === 0) {
      return reply.code(400).send({ error: 'No supported files uploaded.' });
    }

    const job: JobState = {
      id: jobId,
      status: 'Converting',
      result: null,
      inputFolder: inputDir,
      outputFolder: outputDir,
      childProcess: null,
    };
    jobs.set(jobId, job);

    runAgentReadyCli(inputDir, outputDir, (child) => {
      job.childProcess = child;
    })
      .then((result) => {
        job.childProcess = null;
        if (job.status !== 'Cancelling') {
          job.status = 'Completed';
          job.result = result;
        }
      })
      .catch((err) => {
        job.childProcess = null;
        if (job.status !== 'Cancelling') {
          console.error('CLI error:', err);
          job.status = 'Failed';
        }
      });

    return reply.send({ id: jobId, status: 'Converting' });
  });

  // GET /api/jobs/:jobId - Poll status
  fastify.get('/api/jobs/:jobId', async (request: FastifyRequest<{ Params: { jobId: string } }>, reply: FastifyReply) => {
    const job = jobs.get(request.params.jobId);
    if (!job) {
      return reply.code(404).send({ error: 'Job not found' });
    }
    return reply.send({
      id: job.id,
      status: job.status,
      result: job.result,
    });
  });

  // POST /api/jobs/:jobId/cancel - Cancel a running job
  fastify.post('/api/jobs/:jobId/cancel', async (request: FastifyRequest<{ Params: { jobId: string } }>, reply: FastifyReply) => {
    const job = jobs.get(request.params.jobId);
    if (!job) {
      return reply.code(404).send({ error: 'Job not found' });
    }

    if (job.status !== 'Converting') {
      return reply.code(400).send({ error: 'Job is not running', status: job.status });
    }

    job.status = 'Cancelling';

    if (job.childProcess && !job.childProcess.killed) {
      job.childProcess.kill('SIGTERM');
    }

    return reply.send({ id: job.id, status: 'Cancelling' });
  });

  // GET /api/jobs/:jobId/download - Download the generated ZIP
  fastify.get('/api/jobs/:jobId/download', async (request: FastifyRequest<{ Params: { jobId: string } }>, reply: FastifyReply) => {
    const job = jobs.get(request.params.jobId);
    if (!job || job.status !== 'Completed' || !job.result) {
      return reply.code(404).send({ error: 'Job not ready or not found' });
    }

    const zipPath = job.result.export_zip;
    if (!fs.existsSync(zipPath)) {
      return reply.code(404).send({ error: 'Zip file not found' });
    }

    const stream = fs.createReadStream(zipPath);
    reply.header('Content-Type', 'application/zip');
    reply.header('Content-Disposition', `attachment; filename="AgentReady-Export-${job.id}.zip"`);
    return reply.send(stream);
  });

  // GET /api/jobs/:jobId/files/:fileId/preview - Get rendered markdown text
  fastify.get('/api/jobs/:jobId/files/:fileId/preview', async (request: FastifyRequest<{ Params: { jobId: string; fileId: string } }>, reply: FastifyReply) => {
    const { jobId, fileId } = request.params;
    const job = jobs.get(jobId);

    if (!job || job.status !== 'Completed' || !job.result) {
      return reply.code(404).send({ error: 'Job not ready or not found' });
    }

    const fileNode = job.result.files.find(f => f.output_file && f.output_file.includes(fileId));
    if (!fileNode || !fileNode.output_file) {
      return reply.code(404).send({ error: 'File preview not available' });
    }

    const mdPath = path.join(job.outputFolder, fileNode.output_file);
    if (!fs.existsSync(mdPath)) {
      return reply.code(404).send({ error: 'Markdown file missing' });
    }

    const content = fs.readFileSync(mdPath, 'utf8');
    reply.header('Content-Type', 'text/plain; charset=utf-8');
    return reply.send(content);
  });

  // DELETE /api/jobs/:jobId - Cleanup
  fastify.delete('/api/jobs/:jobId', async (request: FastifyRequest<{ Params: { jobId: string } }>, reply: FastifyReply) => {
    const job = jobs.get(request.params.jobId);
    if (job) {
      if (job.childProcess && !job.childProcess.killed) {
        job.childProcess.kill('SIGTERM');
      }
      try {
        const baseDir = path.join(os.tmpdir(), `agentready-${job.id}`);
        fs.rmSync(baseDir, { recursive: true, force: true });
      } catch (err) {
        console.error('Cleanup error:', err);
      }
      jobs.delete(job.id);
    }
    return reply.send({ success: true });
  });
}
