#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const https = require("https");
const { spawn } = require("child_process");

const PACKAGE_JSON_PATH = path.join(__dirname, "..", "package.json");
const PACKAGE_JSON = JSON.parse(fs.readFileSync(PACKAGE_JSON_PATH, "utf8"));
const RELEASE_BASE = `https://github.com/MichengLiang/docutouch/releases/download/v${PACKAGE_JSON.version}`;

function resolveAsset() {
  if (process.platform === "win32" && process.arch === "x64") {
    return {
      assetName: "docutouch-x86_64-pc-windows-msvc.exe",
      cacheName: "docutouch.exe",
    };
  }

  if (process.platform === "linux" && process.arch === "x64") {
    return {
      assetName: "docutouch-x86_64-unknown-linux-gnu",
      cacheName: "docutouch",
    };
  }

  throw new Error(
    `@michengliang/docutouch currently supports Windows x64 and Linux x64 only; got ${process.platform}/${process.arch}`,
  );
}

function cachePathFor(asset) {
  return path.join(
    __dirname,
    "..",
    "vendor",
    `${process.platform}-${process.arch}`,
    asset.cacheName,
  );
}

function ensureDirectory(filePath) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
}

function download(url, destination, redirectCount = 0) {
  if (redirectCount > 5) {
    return Promise.reject(new Error(`Too many redirects while downloading ${url}`));
  }

  return new Promise((resolve, reject) => {
    const request = https.get(url, (response) => {
      const status = response.statusCode || 0;

      if (status >= 300 && status < 400 && response.headers.location) {
        response.resume();
        resolve(download(response.headers.location, destination, redirectCount + 1));
        return;
      }

      if (status !== 200) {
        response.resume();
        reject(new Error(`Failed to download ${url}: HTTP ${status}`));
        return;
      }

      const tempPath = `${destination}.tmp`;
      const file = fs.createWriteStream(tempPath, { mode: 0o755 });

      file.on("error", (error) => {
        response.destroy(error);
      });

      response.on("error", (error) => {
        file.destroy(error);
      });

      file.on("finish", () => {
        file.close((closeError) => {
          if (closeError) {
            reject(closeError);
            return;
          }
          try {
            fs.renameSync(tempPath, destination);
            if (process.platform !== "win32") {
              fs.chmodSync(destination, 0o755);
            }
            resolve();
          } catch (error) {
            reject(error);
          }
        });
      });

      file.on("close", () => {
        if (!fs.existsSync(destination) && fs.existsSync(tempPath)) {
          fs.rmSync(tempPath, { force: true });
        }
      });

      response.pipe(file);
    });

    request.on("error", reject);
  });
}

async function ensureBinary() {
  const asset = resolveAsset();
  const binaryPath = cachePathFor(asset);
  if (!fs.existsSync(binaryPath)) {
    ensureDirectory(binaryPath);
    const url = `${RELEASE_BASE}/${asset.assetName}`;
    await download(url, binaryPath);
  }
  return binaryPath;
}

async function main() {
  try {
    const binaryPath = await ensureBinary();
    const child = spawn(binaryPath, process.argv.slice(2), {
      stdio: "inherit",
    });

    child.on("error", (error) => {
      console.error(error.message);
      process.exit(1);
    });

    child.on("exit", (code, signal) => {
      if (signal) {
        process.kill(process.pid, signal);
        return;
      }
      process.exit(code === null ? 1 : code);
    });
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

void main();
