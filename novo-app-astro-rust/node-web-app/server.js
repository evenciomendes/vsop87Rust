const http = require('http');
const fs = require('fs');
const path = require('path');
const { execFile } = require('child_process');
const url = require('url');
const fallbackEngine = require('./vsop87_fallback');

const PORT = process.env.PORT || 3030;

function getRustBinaryPath() {
  if (process.env.RUST_ASTRO_BIN && fs.existsSync(process.env.RUST_ASTRO_BIN)) {
    return process.env.RUST_ASTRO_BIN;
  }
  const binName = process.platform === 'win32' ? 'rust-astro-engine.exe' : 'rust-astro-engine';
  const releaseBin = path.join(__dirname, '..', 'rust-astro-engine', 'target', 'release', binName);
  const debugBin = path.join(__dirname, '..', 'rust-astro-engine', 'target', 'debug', binName);
  const localBin = path.join(__dirname, 'bin', binName);
  if (fs.existsSync(localBin)) return localBin;
  if (fs.existsSync(releaseBin)) return releaseBin;
  if (fs.existsSync(debugBin)) return debugBin;
  return releaseBin;
}

const server = http.createServer((req, res) => {
  const parsedUrl = url.parse(req.url, true);

  if (parsedUrl.pathname === '/api/calculate') {
    const iso = parsedUrl.query.iso || new Date().toISOString();
    const lng = parsedUrl.query.lng || '-46.6333';
    const dateObj = new Date(iso);
    const unixMs = isNaN(dateObj.getTime()) ? Date.now() : dateObj.getTime();

    const rustBin = getRustBinaryPath();
    if (fs.existsSync(rustBin)) {
      const t0 = process.hrtime.bigint();
      execFile(rustBin, ['--unix-ms', String(unixMs), '--lng', String(lng)], { timeout: 5000 }, (err, stdout) => {
        const nodeTotalMs = Number(process.hrtime.bigint() - t0) / 1e6;
        if (!err) {
          try {
            const data = JSON.parse(stdout);
            data.isoDate = new Date(unixMs).toISOString();
            data.nodeRoundtripMs = Number(nodeTotalMs.toFixed(2));
            res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8', 'Access-Control-Allow-Origin': '*' });
            res.end(JSON.stringify(data));
            return;
          } catch (_) {}
        }
        // Fallback se houver erro
        const data = fallbackEngine.computeAll(unixMs, Number(lng));
        data.isoDate = new Date(unixMs).toISOString();
        res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8', 'Access-Control-Allow-Origin': '*' });
        res.end(JSON.stringify(data));
      });
      return;
    }

    // Se o binário Rust ainda não foi compilado (ex: cargo não instalado), responde com o motor espelho!
    const data = fallbackEngine.computeAll(unixMs, Number(lng));
    data.isoDate = new Date(unixMs).toISOString();
    res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8', 'Access-Control-Allow-Origin': '*' });
    res.end(JSON.stringify(data));
    return;
  }

  if (parsedUrl.pathname === '/' || parsedUrl.pathname === '/index.html') {
    const htmlPath = path.join(__dirname, 'public', 'index.html');
    fs.readFile(htmlPath, 'utf8', (err, content) => {
      if (err) {
        res.writeHead(500);
        res.end('Erro ao carregar index.html');
        return;
      }
      res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
      res.end(content);
    });
    return;
  }

  res.writeHead(404);
  res.end('Not Found');
});

if (require.main === module) {
  server.listen(PORT, () => {
    const rustBin = getRustBinaryPath();
    console.log('==============================================================');
    console.log(' Servidor Web Astrologia (Node.js + Motor Rust VSOP87) no ar!');
    console.log(' Acesse no navegador: http://localhost:' + PORT);
    console.log(' Binário Rust: ' + (fs.existsSync(rustBin) ? 'ATIVO (' + rustBin + ')' : 'Aguardando compilação cargo (usando espelho VSOP87 integrado)'));
    console.log('==============================================================');
  });
}

module.exports = { server };
