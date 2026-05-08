#!/usr/bin/env node
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const STATE_DIR = path.join(ROOT, "tmp", "dev-manager");
const CURRENT_ENV_FILE = path.join(STATE_DIR, "current-env");
const DEFAULT_ENV = "development";

const SERVICES = {
  api: {
    command: "cargo",
    args: ["run", "-p", "msm-app"],
    logFile: "api.log",
    pidFile: "api.pid",
  },
  web: {
    command: process.execPath,
    args: [
      path.join(ROOT, "node_modules", "vite", "bin", "vite.js"),
      "--host",
      "127.0.0.1",
    ],
    cwd: path.join(ROOT, "apps", "web"),
    logFile: "web.log",
    pidFile: "web.pid",
  },
};

function usage(exitCode = 0) {
  const serviceNames = Object.keys(SERVICES).join("|");
  console.log(`MSM development manager

Usage:
  node scripts/dev-manager.mjs start [${serviceNames}|all]
  node scripts/dev-manager.mjs stop [${serviceNames}|all]
  node scripts/dev-manager.mjs restart [${serviceNames}|all]
  node scripts/dev-manager.mjs status
  node scripts/dev-manager.mjs env current
  node scripts/dev-manager.mjs env list
  node scripts/dev-manager.mjs env init <name>
  node scripts/dev-manager.mjs env use <name>

Examples:
  npm run dev:env -- init development
  npm run dev:env -- use development
  npm run dev:start
  npm run dev:status
  npm run dev:stop
`);
  process.exit(exitCode);
}

function ensureStateDir() {
  fs.mkdirSync(STATE_DIR, { recursive: true });
}

function normalizeTarget(target = "all") {
  if (target === "all") {
    return Object.keys(SERVICES);
  }
  if (!SERVICES[target]) {
    throw new Error(
      `Unknown service "${target}". Expected one of: ${Object.keys(SERVICES).join(", ")}, all`,
    );
  }
  return [target];
}

function currentEnvName() {
  ensureStateDir();
  if (!fs.existsSync(CURRENT_ENV_FILE)) {
    fs.writeFileSync(CURRENT_ENV_FILE, `${DEFAULT_ENV}\n`);
  }
  return fs.readFileSync(CURRENT_ENV_FILE, "utf8").trim() || DEFAULT_ENV;
}

function setCurrentEnv(name) {
  ensureStateDir();
  fs.writeFileSync(CURRENT_ENV_FILE, `${name}\n`);
}

function assertEnvName(name) {
  if (!/^[A-Za-z0-9_-]+$/.test(name)) {
    throw new Error("Environment names may only contain letters, numbers, underscores, and dashes.");
  }
}

function envFileFor(name) {
  assertEnvName(name);
  return path.join(ROOT, `.env.${name}`);
}

function envExampleFor(name) {
  assertEnvName(name);
  return path.join(ROOT, `.env.${name}.example`);
}

function initEnv(name) {
  const target = envFileFor(name);
  if (fs.existsSync(target)) {
    console.log(`${path.basename(target)} already exists`);
    return;
  }
  const example = envExampleFor(name);
  if (!fs.existsSync(example)) {
    throw new Error(
      `Missing ${path.basename(example)}. Create it before initializing this environment.`,
    );
  }
  fs.copyFileSync(example, target);
  console.log(`Created ${path.basename(target)} from ${path.basename(example)}`);
}

function listEnvs() {
  const files = fs.readdirSync(ROOT);
  const envs = new Set();
  for (const file of files) {
    const match = /^\.env\.([^.]+)(?:\.example)?$/.exec(file);
    if (match && match[1] !== "local") {
      envs.add(match[1]);
    }
  }
  return [...envs].sort();
}

function parseEnvFile(filePath) {
  if (!fs.existsSync(filePath)) {
    return {};
  }
  const parsed = {};
  const lines = fs.readFileSync(filePath, "utf8").split(/\r?\n/);
  for (const rawLine of lines) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }
    const index = line.indexOf("=");
    if (index === -1) {
      continue;
    }
    const key = line.slice(0, index).trim();
    let value = line.slice(index + 1).trim();
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }
    parsed[key] = value;
  }
  return parsed;
}

function rootRelativePath(value) {
  return path.isAbsolute(value) ? value : path.join(ROOT, value);
}

function sqliteFilePath(databaseUrl) {
  if (!databaseUrl?.startsWith("sqlite:")) {
    return null;
  }
  const filePath = databaseUrl.slice("sqlite:".length);
  if (!filePath || filePath === ":memory:") {
    return null;
  }
  return rootRelativePath(filePath);
}

function ensureLocalRuntimePaths(values) {
  const databasePath = sqliteFilePath(values.MSM_DATABASE_URL);
  if (databasePath) {
    fs.mkdirSync(path.dirname(databasePath), { recursive: true });
  }
  for (const key of ["MSM_ASSET_DIR", "MSM_PREPARED_MEDIA_DIR"]) {
    if (values[key]) {
      fs.mkdirSync(rootRelativePath(values[key]), { recursive: true });
    }
  }
}

function loadEnv() {
  const envName = currentEnvName();
  const envPath = envFileFor(envName);
  if (!fs.existsSync(envPath)) {
    const example = envExampleFor(envName);
    if (fs.existsSync(example)) {
      initEnv(envName);
    } else {
      throw new Error(`Missing ${path.basename(envPath)} and ${path.basename(example)}`);
    }
  }
  return {
    envName,
    values: {
      ...process.env,
      ...parseEnvFile(envPath),
      ...parseEnvFile(path.join(ROOT, ".env.local")),
    },
  };
}

function pidPath(service) {
  return path.join(STATE_DIR, SERVICES[service].pidFile);
}

function logPath(service) {
  return path.join(STATE_DIR, SERVICES[service].logFile);
}

function readPid(service) {
  const file = pidPath(service);
  if (!fs.existsSync(file)) {
    return null;
  }
  const pid = Number.parseInt(fs.readFileSync(file, "utf8").trim(), 10);
  return Number.isFinite(pid) ? pid : null;
}

function isRunning(pid) {
  if (!pid) {
    return false;
  }
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function removePid(service) {
  fs.rmSync(pidPath(service), { force: true });
}

function spawnDetached(command, args, options) {
  if (process.platform === "win32" && /\.(?:cmd|bat)$/i.test(command)) {
    return spawn(process.env.ComSpec ?? "cmd.exe", ["/d", "/c", command, ...args], options);
  }
  return spawn(command, args, options);
}

function startService(service) {
  ensureStateDir();
  const existingPid = readPid(service);
  if (isRunning(existingPid)) {
    console.log(`${service}: already running (pid ${existingPid})`);
    return false;
  }
  removePid(service);

  const { envName, values } = loadEnv();
  ensureLocalRuntimePaths(values);
  const config = SERVICES[service];
  const log = fs.openSync(logPath(service), "a");
  let child;
  try {
    child = spawnDetached(config.command, config.args, {
      cwd: config.cwd ?? ROOT,
      env: values,
      detached: true,
      stdio: ["ignore", log, log],
      windowsHide: true,
    });
  } finally {
    fs.closeSync(log);
  }

  child.unref();
  fs.writeFileSync(pidPath(service), `${child.pid}${os.EOL}`);
  console.log(
    `${service}: started pid ${child.pid} using ${envName} (${path.relative(ROOT, logPath(service))})`,
  );
  return true;
}

function stopService(service) {
  const pid = readPid(service);
  if (!pid) {
    console.log(`${service}: not running`);
    return;
  }
  if (!isRunning(pid)) {
    removePid(service);
    console.log(`${service}: stale pid removed`);
    return;
  }

  if (process.platform === "win32") {
    spawnSync("taskkill", ["/PID", String(pid), "/T", "/F"], {
      stdio: "ignore",
      windowsHide: true,
    });
    removePid(service);
  } else {
    try {
      process.kill(-pid, "SIGTERM");
    } catch {
      process.kill(pid, "SIGTERM");
    }
    removePid(service);
  }
  console.log(`${service}: stopped pid ${pid}`);
}

function status() {
  ensureStateDir();
  console.log(`Environment: ${currentEnvName()}`);
  for (const service of Object.keys(SERVICES)) {
    const pid = readPid(service);
    const running = isRunning(pid);
    if (!running && pid) {
      removePid(service);
    }
    console.log(
      `${service}: ${running ? `running (pid ${pid})` : "stopped"} | log ${path.relative(ROOT, logPath(service))}`,
    );
  }
}

function envCommand(args) {
  const [action, name] = args;
  if (action === "current") {
    console.log(currentEnvName());
    return;
  }
  if (action === "list") {
    const current = currentEnvName();
    for (const envName of listEnvs()) {
      console.log(`${envName === current ? "*" : " "} ${envName}`);
    }
    return;
  }
  if (action === "init") {
    if (!name) {
      throw new Error("env init requires a name");
    }
    initEnv(name);
    return;
  }
  if (action === "use") {
    if (!name) {
      throw new Error("env use requires a name");
    }
    if (!fs.existsSync(envFileFor(name))) {
      initEnv(name);
    }
    setCurrentEnv(name);
    console.log(`Active environment: ${name}`);
    return;
  }
  usage(1);
}

async function main() {
  const [command, targetOrAction, maybeName] = process.argv.slice(2);
  try {
    if (!command || command === "help" || command === "--help" || command === "-h") {
      usage(0);
    }
    if (command === "env") {
      envCommand([targetOrAction, maybeName]);
      return;
    }
    if (command === "status") {
      status();
      return;
    }
    if (command === "start") {
      const started = [];
      for (const service of normalizeTarget(targetOrAction)) {
        try {
          if (startService(service)) {
            started.push(service);
          }
        } catch (error) {
          for (const startedService of started.reverse()) {
            stopService(startedService);
          }
          throw error;
        }
      }
      return;
    }
    if (command === "stop") {
      for (const service of normalizeTarget(targetOrAction)) {
        stopService(service);
      }
      return;
    }
    if (command === "restart") {
      for (const service of normalizeTarget(targetOrAction)) {
        stopService(service);
        startService(service);
      }
      return;
    }
    usage(1);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

await main();
