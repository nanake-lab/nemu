import { defineConfig, type Plugin } from 'vite';
import { resolve } from 'node:path';
import { writeFile, mkdir } from 'node:fs/promises';

function reviewApiPlugin(): Plugin {
  return {
    name: 'review-api-plugin',
    configureServer(server) {
      server.middlewares.use('/api/save-review', async (req, res) => {
        if (req.method !== 'POST') {
          res.statusCode = 405;
          res.end('Method Not Allowed');
          return;
        }

        let body = '';
        req.on('data', (chunk) => {
          body += chunk;
        });

        req.on('end', async () => {
          try {
            const data = JSON.parse(body);
            const dateStr =
              typeof data.date === 'string' && data.date
                ? data.date
                : new Date().toISOString().split('T')[0];
            const projectRoot = resolve(__dirname, '../../');
            const reportsDir = resolve(projectRoot, 'reports');
            const filename = resolve(reportsDir, `review-${dateStr}.md`);
            await mkdir(reportsDir, { recursive: true });
            await writeFile(filename, data.markdown ?? '', 'utf-8');

            res.setHeader('Content-Type', 'application/json');
            res.end(JSON.stringify({ success: true, file: filename }));
          } catch (err) {
            res.statusCode = 500;
            res.setHeader('Content-Type', 'application/json');
            res.end(JSON.stringify({ success: false, error: String(err) }));
          }
        });
      });
    },
  };
}

export default defineConfig({
  base: './',
  plugins: [reviewApiPlugin()],
  build: {
    rollupOptions: {
      input: {
        stage: resolve(__dirname, 'index.html'),
        review: resolve(__dirname, 'review.html'),
      },
    },
  },
});
