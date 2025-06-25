#!/usr/bin/env python3
"""
Mock MCP Server for DAA/QuDAG
This provides a basic MCP server while the real QuDAG MCP builds
"""

import sys
import json
import logging

# Configure logging to stderr only
logging.basicConfig(level=logging.INFO, format='%(message)s', stream=sys.stderr)
logger = logging.getLogger(__name__)

class MockMCPServer:
    def __init__(self):
        self.initialized = False
        
    def handle_request(self, request):
        """Handle JSON-RPC requests"""
        method = request.get("method", "")
        params = request.get("params", {})
        id = request.get("id", None)
        
        # Handle different MCP methods
        if method == "initialize":
            return self.initialize(params, id)
        elif method == "tools/list":
            return self.list_tools(id)
        elif method == "resources/list":
            return self.list_resources(id)
        elif method == "tools/call":
            return self.call_tool(params, id)
        else:
            return self.error_response(id, -32601, f"Method not found: {method}")
    
    def initialize(self, params, id):
        """Handle initialization"""
        self.initialized = True
        return {
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "0.1.0",
                "capabilities": {
                    "tools": {"listChanged": False},
                    "resources": {"listChanged": False, "subscribe": False}
                },
                "serverInfo": {
                    "name": "DAA/QuDAG Mock MCP Server",
                    "version": "0.1.0"
                }
            }
        }
    
    def list_tools(self, id):
        """List available tools"""
        tools = [
            {
                "name": "dag_status",
                "description": "Get DAG consensus status",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "crypto_info",
                "description": "Get quantum crypto information",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "vault_list",
                "description": "List available vaults",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }
        ]
        
        return {
            "jsonrpc": "2.0",
            "id": id,
            "result": {"tools": tools}
        }
    
    def list_resources(self, id):
        """List available resources"""
        resources = [
            {
                "uri": "dag://status",
                "name": "DAG Status",
                "description": "Current DAG consensus status",
                "mimeType": "application/json"
            },
            {
                "uri": "crypto://algorithms",
                "name": "Crypto Algorithms",
                "description": "Available quantum-resistant algorithms",
                "mimeType": "application/json"
            }
        ]
        
        return {
            "jsonrpc": "2.0",
            "id": id,
            "result": {"resources": resources}
        }
    
    def call_tool(self, params, id):
        """Execute a tool"""
        tool_name = params.get("name", "")
        
        if tool_name == "dag_status":
            result = {
                "content": [
                    {
                        "type": "text",
                        "text": json.dumps({
                            "status": "healthy",
                            "vertices": 42,
                            "tips": 3,
                            "consensus": "QR-Avalanche",
                            "message": "Mock DAG is operational"
                        }, indent=2)
                    }
                ]
            }
        elif tool_name == "crypto_info":
            result = {
                "content": [
                    {
                        "type": "text",
                        "text": json.dumps({
                            "algorithms": ["ML-DSA", "ML-KEM", "HQC"],
                            "quantum_resistant": True,
                            "message": "Quantum crypto ready (mock)"
                        }, indent=2)
                    }
                ]
            }
        elif tool_name == "vault_list":
            result = {
                "content": [
                    {
                        "type": "text",
                        "text": json.dumps({
                            "vaults": ["personal", "shared", "backup"],
                            "encrypted": True,
                            "message": "Mock vaults available"
                        }, indent=2)
                    }
                ]
            }
        else:
            return self.error_response(id, -32602, f"Unknown tool: {tool_name}")
        
        return {
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }
    
    def error_response(self, id, code, message):
        """Create error response"""
        return {
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        }
    
    def run(self):
        """Run the server"""
        logger.info("🚀 Mock MCP Server Starting...")
        logger.info("📝 This is a temporary server while QuDAG MCP builds")
        
        while True:
            try:
                # Read JSON-RPC request from stdin
                line = sys.stdin.readline()
                if not line:
                    break
                
                # Parse request
                request = json.loads(line.strip())
                
                # Handle request
                response = self.handle_request(request)
                
                # Send response
                print(json.dumps(response))
                sys.stdout.flush()
                
            except json.JSONDecodeError as e:
                # Send parse error
                error_response = self.error_response(None, -32700, f"Parse error: {e}")
                print(json.dumps(error_response))
                sys.stdout.flush()
            except Exception as e:
                # Send internal error
                error_response = self.error_response(None, -32603, f"Internal error: {e}")
                print(json.dumps(error_response))
                sys.stdout.flush()

if __name__ == "__main__":
    server = MockMCPServer()
    server.run()