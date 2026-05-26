import Fastify from 'fastify';
import multipart from '@fastify/multipart';
import cors from '@fastify/cors';
import jobRoutes from './routes/jobs';

const fastify = Fastify({
  logger: true,
  bodyLimit: 50 * 1024 * 1024, // 50MB
});

async function build() {
  await fastify.register(cors, {
    origin: '*', // For V1 local dev
  });

  // The SDD limits file size to 50MB.
  await fastify.register(multipart, {
    limits: {
      fileSize: 50 * 1024 * 1024, // 50MB limit
    }
  });

  fastify.register(jobRoutes);

  fastify.get('/api/health', async () => {
    return { status: 'ok' };
  });

  return fastify;
}

build().then(app => {
  app.listen({ port: 3001, host: '0.0.0.0' }, (err, address) => {
    if (err) {
      app.log.error(err);
      process.exit(1);
    }
    console.log(`Server listening at ${address}`);
  });
});
