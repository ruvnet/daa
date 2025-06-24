#!/usr/bin/env node

/**
 * Test MCP Connection
 * Tests the QuDAG testnet MCP server connection
 */

const { spawn } = require('child_process');

console.log('Testing QuDAG Testnet MCP Connection...\n');

// Spawn the MCP proxy
const proxy = spawn('node', [
  '/workspaces/QuDAG/.roo/mcp-http-proxy.js',
  'http://109.105.222.156:3333'
]);

let responseCount = 0;

// Handle proxy output
proxy.stdout.on('data', (data) => {
  const response = data.toString().trim();
  if (response) {
    try {
      const json = JSON.parse(response);
      console.log(`Response ${++responseCount}:`, JSON.stringify(json, null, 2));
    } catch (e) {
      console.log('Raw output:', response);
    }
  }
});

proxy.stderr.on('data', (data) => {
  console.error('Proxy:', data.toString().trim());
});

proxy.on('error', (error) => {
  console.error('Failed to start proxy:', error);
  process.exit(1);
});

// Test commands
const testCommands = [
  // Initialize
  { jsonrpc: '2.0', method: 'initialize', params: {}, id: 1 },
  
  // List tools
  { jsonrpc: '2.0', method: 'tools/list', params: {}, id: 2 },
  
  // List resources
  { jsonrpc: '2.0', method: 'resources/list', params: {}, id: 3 },
  
  // Call a tool
  {
    jsonrpc: '2.0',
    method: 'tools/call',
    params: {
      name: 'qudag_dag',
      arguments: { operation: 'get_status' }
    },
    id: 4
  },
  
  // Read a resource
  {
    jsonrpc: '2.0',
    method: 'resources/read',
    params: { uri: 'resource://dag_status' },
    id: 5
  }
];

// Send test commands with delay
let commandIndex = 0;

const sendNextCommand = () => {
  if (commandIndex < testCommands.length) {
    const command = testCommands[commandIndex++];
    console.log(`\nSending command ${command.id}:`, command.method);
    proxy.stdin.write(JSON.stringify(command) + '\n');
    
    setTimeout(sendNextCommand, 1000);
  } else {
    setTimeout(() => {
      console.log('\nTest completed!');
      proxy.kill();
      process.exit(0);
    }, 2000);
  }
};

// Start sending commands after proxy is ready
setTimeout(sendNextCommand, 1000);