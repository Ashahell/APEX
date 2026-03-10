#!/usr/bin/env node

/**
 * Simple test MCP server for E2E testing
 * Implements JSON-RPC over stdio
 */

const readline = require('readline');

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

let requestId = 0;
let initialized = false;

const tools = [
  {
    name: 'echo',
    description: 'Echo back the input message',
    inputSchema: {
      type: 'object',
      properties: {
        message: { type: 'string', description: 'Message to echo' }
      },
      required: ['message']
    }
  },
  {
    name: 'add',
    description: 'Add two numbers',
    inputSchema: {
      type: 'object',
      properties: {
        a: { type: 'number', description: 'First number' },
        b: { type: 'number', description: 'Second number' }
      },
      required: ['a', 'b']
    }
  },
  {
    name: 'get_time',
    description: 'Get current server time',
    inputSchema: {
      type: 'object',
      properties: {}
    }
  }
];

function sendResponse(id, result) {
  const response = {
    jsonrpc: '2.0',
    id: id,
    result: result
  };
  console.log(JSON.stringify(response));
}

function sendError(id, code, message) {
  const error = {
    jsonrpc: '2.0',
    id: id,
    error: {
      code: code,
      message: message
    }
  };
  console.log(JSON.stringify(error));
}

async function handleRequest(data) {
  try {
    const msg = JSON.parse(data);
    
    if (!msg.jsonrpc || msg.jsonrpc !== '2.0') {
      if (msg.id !== undefined) {
        sendError(msg.id, -32600, 'Invalid JSON-RPC');
      }
      return;
    }

    const { id, method, params } = msg;

    switch (method) {
      case 'initialize':
        const result = {
          protocolVersion: '2024-11-05',
          capabilities: {
            tools: {}
          },
          serverInfo: {
            name: 'test-mcp-server',
            version: '1.0.0'
          }
        };
        sendResponse(id, result);
        initialized = true;
        break;

      case 'notifications/initialized':
        // Notification, no response needed
        console.error('[MCP] Server initialized');
        break;

      case 'tools/list':
        if (!initialized) {
          sendError(id, -32000, 'Server not initialized');
          return;
        }
        sendResponse(id, tools);
        break;

      case 'tools/call':
        if (!initialized) {
          sendError(id, -32000, 'Server not initialized');
          return;
        }
        
        const toolName = params.name;
        const args = params.arguments || {};
        let toolResult;
        
        switch (toolName) {
          case 'echo':
            toolResult = {
              success: true,
              content: `Echo: ${args.message || '(no message)'}`
            };
            break;
          case 'add':
            const sum = (args.a || 0) + (args.b || 0);
            toolResult = {
              success: true,
              content: `Result: ${sum}`
            };
            break;
          case 'get_time':
            toolResult = {
              success: true,
              content: `Server time: ${new Date().toISOString()}`
            };
            break;
          default:
            sendError(id, -32601, `Unknown tool: ${toolName}`);
            return;
        }
        
        sendResponse(id, toolResult);
        break;

      default:
        sendError(id, -32601, `Unknown method: ${method}`);
    }
  } catch (e) {
    console.error('[MCP] Error:', e.message);
    if (msg && msg.id !== undefined) {
      sendError(msg.id, -32603, 'Internal error: ' + e.message);
    }
  }
}

console.error('[MCP] Test server starting...');

rl.on('line', (line) => {
  if (line.trim()) {
    handleRequest(line);
  }
});

process.on('SIGTERM', () => {
  console.error('[MCP] Server shutting down');
  process.exit(0);
});
