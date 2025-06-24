import * as fs from 'fs-extra';
import * as path from 'path';
import * as os from 'os';
import axios from 'axios';
import * as tar from 'tar';
import ProgressBar from 'progress';
import { createWriteStream } from 'fs';
import { pipeline } from 'stream/promises';
import { platform as getPlatform, arch as getArch } from 'os';

// Binary version - should match the Rust crate version
const BINARY_VERSION = 'v1.0.2';
const GITHUB_REPO = 'ruvnet/QuDAG';

// Platform mapping
const PLATFORM_MAP: Record<string, string> = {
  darwin: 'apple-darwin',
  linux: 'unknown-linux-gnu',
  win32: 'pc-windows-msvc'
};

// Architecture mapping
const ARCH_MAP: Record<string, string> = {
  x64: 'x86_64',
  arm64: 'aarch64'
};

/**
 * Get the platform-specific binary name
 */
function getBinaryName(): string {
  return getPlatform() === 'win32' ? 'qudag.exe' : 'qudag';
}

/**
 * Get the target triple for the current platform
 */
function getTargetTriple(): string {
  const platform = getPlatform();
  const arch = getArch();
  
  const mappedPlatform = PLATFORM_MAP[platform];
  const mappedArch = ARCH_MAP[arch];
  
  if (!mappedPlatform || !mappedArch) {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }
  
  return `${mappedArch}-${mappedPlatform}`;
}

/**
 * Get the binary directory path
 */
function getBinaryDir(): string {
  return path.join(__dirname, '..', 'bin', 'platform');
}

/**
 * Get the binary path for the current platform
 */
export function getBinaryPath(): string {
  const binaryDir = getBinaryDir();
  const binaryName = getBinaryName();
  return path.join(binaryDir, binaryName);
}

/**
 * Check if the binary is already installed
 */
function isBinaryInstalled(): boolean {
  const binaryPath = getBinaryPath();
  return fs.existsSync(binaryPath);
}

/**
 * Download the binary for the current platform
 */
async function downloadBinary(): Promise<void> {
  const targetTriple = getTargetTriple();
  const binaryName = getBinaryName();
  const archiveName = `qudag-${BINARY_VERSION}-${targetTriple}.tar.gz`;
  
  // Construct the download URL
  const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/${BINARY_VERSION}/${archiveName}`;
  
  console.log(`Downloading QuDAG binary for ${targetTriple}...`);
  console.log(`URL: ${downloadUrl}`);
  
  try {
    // Create binary directory
    const binaryDir = getBinaryDir();
    await fs.ensureDir(binaryDir);
    
    // Download the archive
    const response = await axios({
      method: 'get',
      url: downloadUrl,
      responseType: 'stream',
      timeout: 30000,
      headers: {
        'User-Agent': 'qudag-npm'
      }
    });
    
    const totalLength = parseInt(response.headers['content-length'] || '0', 10);
    
    // Create progress bar
    const progressBar = new ProgressBar('Downloading [:bar] :percent :etas', {
      width: 40,
      complete: '=',
      incomplete: ' ',
      renderThrottle: 100,
      total: totalLength
    });
    
    // Update progress
    response.data.on('data', (chunk: Buffer) => {
      progressBar.tick(chunk.length);
    });
    
    // Save to temporary file
    const tempFile = path.join(os.tmpdir(), archiveName);
    const writer = createWriteStream(tempFile);
    
    await pipeline(response.data, writer);
    
    console.log('\nExtracting binary...');
    
    // Extract the archive
    await tar.extract({
      file: tempFile,
      cwd: binaryDir,
      filter: (path) => path.endsWith(binaryName)
    });
    
    // Make the binary executable on Unix-like systems
    if (getPlatform() !== 'win32') {
      const binaryPath = getBinaryPath();
      await fs.chmod(binaryPath, 0o755);
    }
    
    // Clean up temporary file
    await fs.remove(tempFile);
    
    console.log('QuDAG binary installed successfully!');
    
  } catch (error: any) {
    if (error.response?.status === 404) {
      throw new Error(
        `Binary not found for platform ${targetTriple}. ` +
        `Please check if a release exists for your platform at ` +
        `https://github.com/${GITHUB_REPO}/releases`
      );
    }
    throw new Error(`Failed to download binary: ${error.message}`);
  }
}

/**
 * Ensure the binary is installed, downloading if necessary
 */
export async function ensureBinary(): Promise<void> {
  if (!isBinaryInstalled()) {
    await downloadBinary();
  }
}

/**
 * Get information about the current platform
 */
export function getPlatformInfo(): {
  platform: string;
  arch: string;
  targetTriple: string;
  binaryName: string;
  binaryPath: string;
} {
  return {
    platform: getPlatform(),
    arch: getArch(),
    targetTriple: getTargetTriple(),
    binaryName: getBinaryName(),
    binaryPath: getBinaryPath()
  };
}