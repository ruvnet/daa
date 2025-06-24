/**
 * QuDAG NPM Package Installation Script
 * This script runs during npm install to download the appropriate binary
 */

import { ensureBinary, getPlatformInfo } from './binary-manager';
import chalk from 'chalk';

async function install() {
  console.log(chalk.blue.bold('\n🌐 QuDAG Installation\n'));
  
  // Get platform information
  const platformInfo = getPlatformInfo();
  console.log(chalk.gray('Platform Information:'));
  console.log(chalk.gray(`  OS: ${platformInfo.platform}`));
  console.log(chalk.gray(`  Architecture: ${platformInfo.arch}`));
  console.log(chalk.gray(`  Target: ${platformInfo.targetTriple}`));
  console.log(chalk.gray(`  Binary: ${platformInfo.binaryName}\n`));
  
  try {
    // Ensure binary is installed
    await ensureBinary();
    
    console.log(chalk.green.bold('\n✅ QuDAG installation completed successfully!\n'));
    console.log(chalk.gray('You can now use QuDAG with:'));
    console.log(chalk.cyan('  npx qudag --help'));
    console.log(chalk.cyan('  qudag --help') + chalk.gray(' (if installed globally)\n'));
    
  } catch (error: any) {
    console.error(chalk.red.bold('\n❌ Installation failed:'), error.message);
    console.error(chalk.yellow('\nPlease try the following:'));
    console.error(chalk.gray('1. Check your internet connection'));
    console.error(chalk.gray('2. Verify that your platform is supported'));
    console.error(chalk.gray('3. Check if a release exists for your platform at:'));
    console.error(chalk.blue('   https://github.com/ruvnet/QuDAG/releases\n'));
    
    // Don't fail the npm install - allow the package to be installed
    // The binary will be downloaded on first use if needed
    console.log(chalk.yellow('⚠️  Warning: Binary not downloaded. It will be downloaded on first use.\n'));
  }
}

// Run installation
if (require.main === module) {
  install().catch((err) => {
    console.error('Unexpected error during installation:', err);
    process.exit(0); // Don't fail npm install
  });
}