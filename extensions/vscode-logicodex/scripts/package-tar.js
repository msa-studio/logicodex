const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const root = path.resolve(__dirname, '..');
const packageJson = require(path.join(root, 'package.json'));
const distDir = path.join(root, 'dist');
const packageName = `${packageJson.name}-${packageJson.version}`;
const stagingRoot = path.join(distDir, packageName);
const archivePath = path.join(distDir, `${packageName}.tar.gz`);

const include = [
  'package.json',
  'README.md',
  'language-configuration.json',
  'out',
  'resources',
  'snippets',
  'syntaxes',
  'examples'
];

function copyRecursive(src, dest) {
  const stat = fs.statSync(src);
  if (stat.isDirectory()) {
    fs.mkdirSync(dest, { recursive: true });
    for (const entry of fs.readdirSync(src)) {
      copyRecursive(path.join(src, entry), path.join(dest, entry));
    }
    return;
  }
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.copyFileSync(src, dest);
}

fs.rmSync(distDir, { recursive: true, force: true });
fs.mkdirSync(stagingRoot, { recursive: true });

for (const item of include) {
  const src = path.join(root, item);
  if (fs.existsSync(src)) {
    copyRecursive(src, path.join(stagingRoot, item));
  }
}

execFileSync('tar', ['-czf', archivePath, '-C', distDir, packageName], { stdio: 'inherit' });
console.log(archivePath);
