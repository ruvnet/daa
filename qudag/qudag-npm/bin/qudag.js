#!/usr/bin/env node

/**
 * QuDAG CLI Entry Point
 * This script serves as the entry point for the QuDAG CLI when installed via NPM.
 * It ensures the binary is installed and delegates to the actual QuDAG binary.
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Import the binary manager
const { ensureBinary, getBinaryPath } = require('../dist/binary-manager');

async function main() {
  try {
    // Ensure the binary is installed
    await ensureBinary();
    
    // Get the path to the binary
    const binaryPath = getBinaryPath();
    
    // Check if binary exists
    if (!fs.existsSync(binaryPath)) {
      console.error('QuDAG binary not found. Please reinstall the package.');
      process.exit(1);
    }
    
    // Forward all arguments to the actual binary
    const args = process.argv.slice(2);
    
    // Spawn the binary with inherited stdio
    const child = spawn(binaryPath, args, {
      stdio: 'inherit',
      env: process.env
    });
    
    // Forward the exit code
    child.on('exit', (code) => {
      process.exit(code || 0);
    });
    
    // Handle errors
    child.on('error', (err) => {
      console.error('Failed to execute QuDAG binary:', err.message);
      process.exit(1);
    });
    
    // Handle signals
    process.on('SIGINT', () => {
      child.kill('SIGINT');
    });
    
    process.on('SIGTERM', () => {
      child.kill('SIGTERM');
    });
    
  } catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
  }
}

// Run the main function
main().catch((err) => {
  console.error('Unexpected error:', err);
  process.exit(1);
});