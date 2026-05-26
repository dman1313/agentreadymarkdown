import { FastifyInstance, FastifyRequest, FastifyReply } from 'fastify';
import { v4 as uuidv4 } from 'uuid';
import fs from 'fs';
import path from 'path';
import os from 'os';
import util from 'util';
import { pipeline } from 'stream';
import { runAgentReadyCli, JobResult } from '../services/cli';

const pump = util.promisify(pipeline);

// In-memory job state for V1
interface JobState {
  id: string;
  status: 'Converting' | 'Completed' | 'Failed';
  result: JobResult | null;
  inputFolder: string;
  outputFolder: string;
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

    // Create directories
    fs.mkdirSync(inputDir, { recursive: true });

    let hasFiles = false;
    for await (const part of parts) {
      if (part.type === 'file') {
        hasFiles = true;
        const savePath = path.join(inputDir, part.filename);
        await pump(part.file, fs.createWriteStream(savePath));
      }
    }

    if (!hasFiles) {
      return reply.code(400).send({ error: "No files uploaded." });
    }

    const job: JobState = {
      id: jobId,
      status: 'Converting',
      result: null,
      inputFolder: inputDir,
      outputFolder: outputDir,
    };
    jobs.set(jobId, job);

    // Run conversion async
    runAgentReadyCli(inputDir, outputDir)
      .then((result) => {
        job.status = 'Completed';
        job.result = result;
      })
      .catch((err) => {
        console.error("CLI error:", err);
        job.status = 'Failed';
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
  fastify.get('/api/jobs/:jobId/files/:fileId/preview', async (request: FastifyRequest<{ Params: { jobId: string, fileId: string } }>, reply: FastifyReply) => {
    const { jobId, fileId } = request.params;
    const job = jobs.get(jobId);
    
    if (!job || job.status !== 'Completed' || !job.result) {
      return reply.code(404).send({ error: 'Job not ready or not found' });
    }

    // fileId is encoded path from UI. We just search the job result to ensure it's valid.
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
      try {
        const baseDir = path.join(os.tmpdir(), `agentready-${job.id}`);
        fs.rmSync(baseDir, { recursive: true, force: true });
      } catch (err) {
        console.error("Cleanup error:", err);
      }
      jobs.delete(job.id);
    }
    return reply.send({ success: true });
  });
}
