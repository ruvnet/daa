#!/usr/bin/env node
/**
 * HTTP-to-stdio bridge for DAA MCP Server
 * This server accepts HTTP requests and forwards them to the stdio-based MCP server
 */

import express from 'express';
import cors from 'cors';
import { spawn } from 'child_process';
import { v4 as uuidv4 } from 'uuid';

const app = express();
const PORT = process.env.MCP_BRIDGE_PORT || 3001;

// Enable CORS for dashboard
app.use(cors());
app.use(express.json());

// MCP server process
let mcpProcess = null;
let requestQueue = new Map();
let isInitialized = false;

// Start MCP server process
function startMcpServer() {
  console.log('Starting DAA MCP Server...');
  
  mcpProcess = spawn('/usr/bin/python3', ['/workspaces/daa/daa-mcp-server.py'], {
    stdio: ['pipe', 'pipe', 'pipe']
  });

  // Handle stdout from MCP server
  mcpProcess.stdout.on('data', (data) => {
    try {
      const response = JSON.parse(data.toString());
      const id = response.id;
      
      if (requestQueue.has(id)) {
        const { resolve } = requestQueue.get(id);
        requestQueue.delete(id);
        resolve(response);
      }
    } catch (error) {
      console.error('Error parsing MCP response:', error);
    }
  });

  // Handle stderr from MCP server
  mcpProcess.stderr.on('data', (data) => {
    console.error('MCP Server Error:', data.toString());
  });

  mcpProcess.on('close', (code) => {
    console.log(`MCP Server exited with code ${code}`);
    mcpProcess = null;
    isInitialized = false;
  });

  // Initialize the MCP server
  initializeMcpServer();
}

// Initialize MCP server
async function initializeMcpServer() {
  const initRequest = {
    jsonrpc: "2.0",
    method: "initialize",
    params: {
      protocolVersion: "2024-11-05",
      capabilities: {
        tools: {},
        resources: {}
      },
      clientInfo: {
        name: "daa-dashboard-bridge",
        version: "1.0.0"
      }
    },
    id: uuidv4()
  };

  const response = await sendToMcp(initRequest);
  if (response.result) {
    isInitialized = true;
    console.log('MCP Server initialized successfully');
  }
}

// Send request to MCP server
function sendToMcp(request) {
  return new Promise((resolve, reject) => {
    if (!mcpProcess) {
      reject(new Error('MCP server not running'));
      return;
    }

    const id = request.id || uuidv4();
    request.id = id;

    requestQueue.set(id, { resolve, reject });

    // Send request to MCP server stdin
    mcpProcess.stdin.write(JSON.stringify(request) + '\n');

    // Timeout after 30 seconds
    setTimeout(() => {
      if (requestQueue.has(id)) {
        requestQueue.delete(id);
        reject(new Error('Request timeout'));
      }
    }, 30000);
  });
}

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({
    status: mcpProcess && isInitialized ? 'healthy' : 'unhealthy',
    initialized: isInitialized,
    process: mcpProcess ? 'running' : 'stopped'
  });
});

// Main MCP endpoint for all requests
app.post('/', async (req, res) => {
  try {
    if (!mcpProcess || !isInitialized) {
      throw new Error('MCP server not initialized');
    }

    const response = await sendToMcp(req.body);
    res.json(response);
  } catch (error) {
    res.status(500).json({
      jsonrpc: "2.0",
      error: {
        code: -32603,
        message: error.message
      },
      id: req.body.id || null
    });
  }
});

// SSE endpoint for real-time updates (stub for now)
app.get('/sse', (req, res) => {
  res.setHeader('Content-Type', 'text/event-stream');
  res.setHeader('Cache-Control', 'no-cache');
  res.setHeader('Connection', 'keep-alive');
  
  // Send initial connection event
  res.write('event: connected\ndata: {"status": "connected"}\n\n');
  
  // Keep connection alive
  const keepAlive = setInterval(() => {
    res.write('event: ping\ndata: {"time": "' + new Date().toISOString() + '"}\n\n');
  }, 30000);
  
  req.on('close', () => {
    clearInterval(keepAlive);
  });
});

// Start the bridge server
app.listen(PORT, '127.0.0.1', () => {
  console.log(`DAA MCP HTTP Bridge running on http://127.0.0.1:${PORT}`);
  startMcpServer();
});

// Graceful shutdown
process.on('SIGINT', () => {
  console.log('\nShutting down MCP bridge...');
  if (mcpProcess) {
    mcpProcess.kill();
  }
  process.exit(0);
});