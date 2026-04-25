#!/usr/bin/env node

/**
 * Install Chorus skills to OpenCode global skills directory
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

const skills = [
  'chorus-task',
  'chorus-context',
  'chorus-artifacts',
  'chorus-complete',
  'chorus-party'
];

function getGlobalSkillsDir() {
  const home = os.homedir();
  
  switch (process.platform) {
    case 'win32':
      return path.join(home, '.config', 'opencode', 'skills');
    case 'darwin':
    case 'linux':
    default:
      return path.join(home, '.config', 'opencode', 'skills');
  }
}

function copyRecursive(src, dest) {
  const exists = fs.existsSync(src);
  const stats = exists && fs.statSync(src);
  const isDirectory = exists && stats.isDirectory();
  
  if (isDirectory) {
    if (!fs.existsSync(dest)) {
      fs.mkdirSync(dest, { recursive: true });
    }
    fs.readdirSync(src).forEach(childItem => {
      copyRecursive(path.join(src, childItem), path.join(dest, childItem));
    });
  } else {
    fs.copyFileSync(src, dest);
  }
}

function main() {
  const sourceDir = __dirname;
  const targetDir = getGlobalSkillsDir();
  
  console.log('Installing Chorus skills...');
  console.log(`Target: ${targetDir}\n`);
  
  // Create target directory
  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir, { recursive: true });
    console.log('✓ Created skills directory');
  }
  
  // Install each skill
  let installed = 0;
  for (const skill of skills) {
    const skillSource = path.join(sourceDir, skill);
    const skillTarget = path.join(targetDir, skill);
    
    if (fs.existsSync(skillSource)) {
      copyRecursive(skillSource, skillTarget);
      console.log(`✓ Installed ${skill}`);
      installed++;
    } else {
      console.log(`✗ Missing ${skill}`);
    }
  }
  
  console.log(`\n✓ Installed ${installed}/${skills.length} skills`);
  console.log('\nNext steps:');
  console.log('1. Add skills to your opencode.json permissions');
  console.log('2. Start a Chorus workflow: chorus run <workflow>');
  console.log('3. OpenCode will auto-load relevant skills when Chorus files are detected');
}

main();