/**
 * QuDAG NPM Package Test Script
 * Verifies that the package is correctly installed and functional
 */

import { QuDAG, isInstalled, getPlatformInfo } from './index';
import chalk from 'chalk';

async function runTests() {
  console.log(chalk.blue.bold('\n🧪 QuDAG NPM Package Tests\n'));
  
  // Test 1: Check if binary is installed
  console.log(chalk.yellow('Test 1: Checking if binary is installed...'));
  const installed = isInstalled();
  if (installed) {
    console.log(chalk.green('✓ Binary is installed'));
  } else {
    console.log(chalk.red('✗ Binary is not installed'));
    return false;
  }
  
  // Test 2: Get platform information
  console.log(chalk.yellow('\nTest 2: Getting platform information...'));
  const platformInfo = getPlatformInfo();
  console.log(chalk.green('✓ Platform information retrieved:'));
  console.log(chalk.gray(`  Platform: ${platformInfo.platform}`));
  console.log(chalk.gray(`  Architecture: ${platformInfo.arch}`));
  console.log(chalk.gray(`  Target Triple: ${platformInfo.targetTriple}`));
  console.log(chalk.gray(`  Binary Path: ${platformInfo.binaryPath}`));
  
  // Test 3: Execute help command
  console.log(chalk.yellow('\nTest 3: Executing help command...'));
  try {
    const result = await QuDAG.raw(['--help']);
    if (result.code === 0) {
      console.log(chalk.green('✓ Help command executed successfully'));
      console.log(chalk.gray('\nOutput preview:'));
      const lines = result.stdout.split('\n').slice(0, 5);
      lines.forEach(line => console.log(chalk.gray(`  ${line}`)));
      console.log(chalk.gray('  ...\n'));
    } else {
      console.log(chalk.red('✗ Help command failed'));
      console.error(chalk.red(`Exit code: ${result.code}`));
      console.error(chalk.red(`Error: ${result.stderr}`));
      return false;
    }
  } catch (error: any) {
    console.log(chalk.red('✗ Failed to execute command'));
    console.error(chalk.red(`Error: ${error.message}`));
    return false;
  }
  
  // Test 4: Check version
  console.log(chalk.yellow('Test 4: Checking version...'));
  try {
    const result = await QuDAG.raw(['--version']);
    if (result.code === 0) {
      console.log(chalk.green('✓ Version command executed successfully'));
      console.log(chalk.gray(`  Version: ${result.stdout.trim()}`));
    } else {
      console.log(chalk.red('✗ Version command failed'));
      return false;
    }
  } catch (error: any) {
    console.log(chalk.red('✗ Failed to get version'));
    console.error(chalk.red(`Error: ${error.message}`));
    return false;
  }
  
  return true;
}

// Run tests
if (require.main === module) {
  runTests().then((success) => {
    if (success) {
      console.log(chalk.green.bold('\n✅ All tests passed!\n'));
      process.exit(0);
    } else {
      console.log(chalk.red.bold('\n❌ Some tests failed!\n'));
      process.exit(1);
    }
  }).catch((err) => {
    console.error(chalk.red.bold('\n❌ Test execution failed:'), err);
    process.exit(1);
  });
}