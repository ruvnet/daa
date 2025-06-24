#!/usr/bin/env node

/**
 * MCP HTTP Proxy - Bridges HTTP MCP server to stdio transport
 * Usage: node mcp-http-proxy.js <mcp-server-url>
 */

const http = require('http');
const https = require('https');
const url = require('url');
const readline = require('readline');

const serverUrl = process.argv[2] || 'http://109.105.222.156:3333';
const baseUrl = new url.URL(serverUrl);

// Setup readline for stdio
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

// Helper to make HTTP requests
async function makeRequest(endpoint, method = 'GET', data = null) {
  const options = {
    hostname: baseUrl.hostname,
    port: baseUrl.port || (baseUrl.protocol === 'https:' ? 443 : 80),
    path: `/mcp${endpoint}`,
    method: method,
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    }
  };

  if (data) {
    options.headers['Content-Length'] = Buffer.byteLength(JSON.stringify(data));
  }

  return new Promise((resolve, reject) => {
    const client = baseUrl.protocol === 'https:' ? https : http;
    const req = client.request(options, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        try {
          resolve(JSON.parse(body));
        } catch (e) {
          resolve({ error: 'Invalid JSON response', body });
        }
      });
    });

    req.on('error', reject);
    
    if (data) {
      req.write(JSON.stringify(data));
    }
    req.end();
  });
}

// Handle JSON-RPC requests
async function handleRequest(request) {
  const { method, params, id } = request;

  try {
    let result;
    
    switch (method) {
      case 'initialize':
        const discovery = await makeRequest('');
        result = {
          protocolVersion: '2024-11-05',
          capabilities: discovery.mcp?.capabilities || {},
          serverInfo: discovery.mcp?.serverInfo || {
            name: 'QuDAG MCP Server',
            version: '1.0.0'
          }
        };
        break;

      case 'mcp/list_tools':
      case 'tools/list':
        const tools = await makeRequest('/tools');
        result = tools;
        break;

      case 'mcp/list_resources':
      case 'resources/list':
        const resources = await makeRequest('/resources');
        result = resources;
        break;

      case 'tools/call':
        const toolResult = await makeRequest('/tools/call', 'POST', params);
        result = toolResult;
        break;

      case 'resources/read':
        const resourceName = params.uri?.replace('resource://', '') || params.name;
        const resource = await makeRequest(`/resources/${resourceName}`);
        result = resource;
        break;

      default:
        // Try JSON-RPC endpoint for other methods
        const rpcResult = await makeRequest('/rpc', 'POST', request);
        if (rpcResult.result) {
          result = rpcResult.result;
        } else if (rpcResult.error) {
          throw new Error(rpcResult.error.message || 'RPC error');
        } else {
          throw new Error(`Unknown method: ${method}`);
        }
    }

    return {
      jsonrpc: '2.0',
      result,
      id
    };
  } catch (error) {
    return {
      jsonrpc: '2.0',
      error: {
        code: -32603,
        message: error.message
      },
      id
    };
  }
}

// Main message loop
rl.on('line', async (line) => {
  try {
    const request = JSON.parse(line);
    const response = await handleRequest(request);
    console.log(JSON.stringify(response));
  } catch (error) {
    console.log(JSON.stringify({
      jsonrpc: '2.0',
      error: {
        code: -32700,
        message: 'Parse error'
      },
      id: null
    }));
  }
});

// Send initial capabilities on startup
makeRequest('').then(discovery => {
  console.error(`MCP HTTP Proxy connected to ${serverUrl}`);
  console.error('Capabilities:', JSON.stringify(discovery.mcp?.capabilities || {}));
}).catch(err => {
  console.error(`Failed to connect to MCP server: ${err.message}`);
});