import React, { useState, useEffect } from 'react';
import { 
  Terminal, 
  Play, 
  Square, 
  RotateCcw, 
  Settings, 
  Code, 
  Database,
  Zap,
  AlertCircle,
  CheckCircle,
  Clock,
  Copy,
  Download,
  Trash2,
  ChevronDown,
  ChevronRight
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { daaMcpClient } from '@/lib/mcp-client';
import McpConnectionStatus from './McpConnectionStatus';

interface ToolCall {
  id: string;
  timestamp: Date;
  tool: string;
  arguments: any;
  status: 'pending' | 'success' | 'error';
  result?: any;
  error?: string;
  duration?: number;
}

interface McpToolsDebuggerProps {
  isOpen?: boolean;
  onClose?: () => void;
}

const AVAILABLE_TOOLS = [
  { name: 'daa_status', description: 'Get DAA system status', category: 'System' },
  { name: 'daa_agent_list', description: 'List all agents', category: 'Agents' },
  { name: 'daa_agent_show', description: 'Show agent details', category: 'Agents', params: { agent_id: 'string' } },
  { name: 'daa_agent_create', description: 'Create new agent', category: 'Agents', params: { name: 'string', agent_type: 'string', capabilities: 'string?' } },
  { name: 'daa_agent_stop', description: 'Stop agent', category: 'Agents', params: { agent_id: 'string', force: 'boolean?' } },
  { name: 'daa_agent_restart', description: 'Restart agent', category: 'Agents', params: { agent_id: 'string' } },
  { name: 'daa_config_show', description: 'Show configuration', category: 'Config' },
  { name: 'daa_config_get', description: 'Get config value', category: 'Config', params: { key: 'string' } },
  { name: 'daa_config_set', description: 'Set config value', category: 'Config', params: { key: 'string', value: 'any' } },
  { name: 'daa_network_status', description: 'Get network status', category: 'Network' },
  { name: 'daa_network_peers', description: 'List network peers', category: 'Network' },
  { name: 'daa_network_connect', description: 'Connect to network', category: 'Network', params: { node: 'string?' } },
  { name: 'daa_logs', description: 'Get system logs', category: 'System', params: { lines: 'number?', level: 'string?', component: 'string?' } },
  { name: 'daa_add_rule', description: 'Add automation rule', category: 'Rules', params: { name: 'string', rule_type: 'string', params: 'string?', description: 'string?' } },
  { name: 'daa_init', description: 'Initialize DAA', category: 'System', params: { directory: 'string?', template: 'string?', force: 'boolean?' } },
  { name: 'daa_start', description: 'Start orchestrator', category: 'System', params: { daemon: 'boolean?' } },
  { name: 'daa_stop', description: 'Stop orchestrator', category: 'System', params: { force: 'boolean?' } },
];

export function McpToolsDebugger({ isOpen = false, onClose }: McpToolsDebuggerProps) {
  const [selectedTool, setSelectedTool] = useState<string>('daa_status');
  const [toolArguments, setToolArguments] = useState<string>('{}');
  const [callHistory, setCallHistory] = useState<ToolCall[]>([]);
  const [isExecuting, setIsExecuting] = useState(false);
  const [expandedResults, setExpandedResults] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState<string>('all');

  const selectedToolInfo = AVAILABLE_TOOLS.find(tool => tool.name === selectedTool);

  useEffect(() => {
    // Load call history from localStorage
    const savedHistory = localStorage.getItem('mcp-debug-history');
    if (savedHistory) {
      try {
        const parsed = JSON.parse(savedHistory).map((call: any) => ({
          ...call,
          timestamp: new Date(call.timestamp)
        }));
        setCallHistory(parsed);
      } catch (error) {
        console.error('Failed to load debug history:', error);
      }
    }
  }, []);

  useEffect(() => {
    // Save call history to localStorage
    localStorage.setItem('mcp-debug-history', JSON.stringify(callHistory));
  }, [callHistory]);

  const executeToolCall = async () => {
    if (!selectedTool) return;

    const callId = `call-${Date.now()}`;
    let parsedArguments = {};

    try {
      parsedArguments = JSON.parse(toolArguments);
    } catch (error) {
      addCallToHistory(callId, selectedTool, toolArguments, 'error', undefined, 'Invalid JSON arguments');
      return;
    }

    setIsExecuting(true);
    const startTime = Date.now();

    const newCall: ToolCall = {
      id: callId,
      timestamp: new Date(),
      tool: selectedTool,
      arguments: parsedArguments,
      status: 'pending'
    };

    setCallHistory(prev => [newCall, ...prev.slice(0, 49)]); // Keep last 50 calls

    try {
      const result = await daaMcpClient.callTool(selectedTool, parsedArguments);
      const duration = Date.now() - startTime;
      
      setCallHistory(prev => prev.map(call => 
        call.id === callId 
          ? { ...call, status: 'success', result, duration }
          : call
      ));
    } catch (error) {
      const duration = Date.now() - startTime;
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      setCallHistory(prev => prev.map(call => 
        call.id === callId 
          ? { ...call, status: 'error', error: errorMessage, duration }
          : call
      ));
    } finally {
      setIsExecuting(false);
    }
  };

  const addCallToHistory = (id: string, tool: string, args: any, status: ToolCall['status'], result?: any, error?: string) => {
    const newCall: ToolCall = {
      id,
      timestamp: new Date(),
      tool,
      arguments: args,
      status,
      result,
      error
    };
    setCallHistory(prev => [newCall, ...prev.slice(0, 49)]);
  };

  const clearHistory = () => {
    setCallHistory([]);
    localStorage.removeItem('mcp-debug-history');
  };

  const exportHistory = () => {
    const dataStr = JSON.stringify(callHistory, null, 2);
    const dataUri = 'data:application/json;charset=utf-8,'+ encodeURIComponent(dataStr);
    
    const exportFileDefaultName = `mcp-debug-history-${new Date().toISOString().split('T')[0]}.json`;
    
    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const toggleResultExpansion = (callId: string) => {
    setExpandedResults(prev => {
      const newSet = new Set(prev);
      if (newSet.has(callId)) {
        newSet.delete(callId);
      } else {
        newSet.add(callId);
      }
      return newSet;
    });
  };

  const getStatusIcon = (status: ToolCall['status']) => {
    switch (status) {
      case 'pending': return <Clock className="h-4 w-4 text-yellow-400" />;
      case 'success': return <CheckCircle className="h-4 w-4 text-green-400" />;
      case 'error': return <AlertCircle className="h-4 w-4 text-red-400" />;
    }
  };

  const getStatusColor = (status: ToolCall['status']) => {
    switch (status) {
      case 'pending': return 'text-yellow-400 bg-yellow-400/10';
      case 'success': return 'text-green-400 bg-green-400/10';
      case 'error': return 'text-red-400 bg-red-400/10';
    }
  };

  const filteredHistory = callHistory.filter(call => {
    if (filter === 'all') return true;
    if (filter === 'success') return call.status === 'success';
    if (filter === 'error') return call.status === 'error';
    if (filter === 'pending') return call.status === 'pending';
    return true;
  });

  const groupedTools = AVAILABLE_TOOLS.reduce((acc, tool) => {
    if (!acc[tool.category]) acc[tool.category] = [];
    acc[tool.category].push(tool);
    return acc;
  }, {} as Record<string, typeof AVAILABLE_TOOLS>);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="bg-gray-900 border border-green-500/40 rounded-lg w-full max-w-6xl h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-700">
          <div className="flex items-center space-x-3">
            <Terminal className="h-6 w-6 text-green-400" />
            <h2 className="text-xl font-bold text-green-400">MCP Tools Debugger</h2>
          </div>
          <div className="flex items-center space-x-3">
            <McpConnectionStatus compact />
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-300 p-1"
            >
              ✕
            </button>
          </div>
        </div>

        <div className="flex flex-1 overflow-hidden">
          {/* Tool Selector */}
          <div className="w-1/3 border-r border-gray-700 flex flex-col">
            <div className="p-4 border-b border-gray-700">
              <h3 className="text-lg font-semibold text-green-400 mb-3">Available Tools</h3>
              {Object.entries(groupedTools).map(([category, tools]) => (
                <div key={category} className="mb-4">
                  <h4 className="text-sm font-medium text-gray-400 mb-2 uppercase tracking-wide">
                    {category}
                  </h4>
                  <div className="space-y-1">
                    {tools.map(tool => (
                      <button
                        key={tool.name}
                        onClick={() => {
                          setSelectedTool(tool.name);
                          if (tool.params) {
                            const defaultArgs = Object.entries(tool.params).reduce((acc, [key, type]) => {
                              if (type.includes('string')) acc[key] = '';
                              else if (type.includes('number')) acc[key] = 0;
                              else if (type.includes('boolean')) acc[key] = false;
                              return acc;
                            }, {} as any);
                            setToolArguments(JSON.stringify(defaultArgs, null, 2));
                          } else {
                            setToolArguments('{}');
                          }
                        }}
                        className={`w-full text-left p-2 rounded text-sm transition-colors ${
                          selectedTool === tool.name
                            ? 'bg-green-500/20 text-green-400 border border-green-500/40'
                            : 'text-gray-300 hover:bg-gray-800'
                        }`}
                      >
                        <div className="font-mono">{tool.name}</div>
                        <div className="text-xs text-gray-400 mt-1">{tool.description}</div>
                      </button>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Tool Execution */}
          <div className="w-2/3 flex flex-col">
            <div className="p-4 border-b border-gray-700">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-green-400">
                  Execute: {selectedTool}
                </h3>
                <button
                  onClick={executeToolCall}
                  disabled={isExecuting}
                  className="flex items-center space-x-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 text-white px-4 py-2 rounded-lg transition-colors"
                >
                  {isExecuting ? (
                    <>
                      <Square className="h-4 w-4" />
                      <span>Executing...</span>
                    </>
                  ) : (
                    <>
                      <Play className="h-4 w-4" />
                      <span>Execute</span>
                    </>
                  )}
                </button>
              </div>

              {selectedToolInfo?.description && (
                <p className="text-gray-400 text-sm mb-4">{selectedToolInfo.description}</p>
              )}

              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-400 mb-2">
                    Arguments (JSON)
                  </label>
                  <textarea
                    value={toolArguments}
                    onChange={(e) => setToolArguments(e.target.value)}
                    className="w-full h-32 px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-green-400 font-mono text-sm focus:border-green-500 focus:outline-none"
                    placeholder="Enter JSON arguments..."
                  />
                </div>

                {selectedToolInfo?.params && (
                  <div className="bg-gray-800/50 border border-gray-700 rounded-lg p-3">
                    <h4 className="text-sm font-medium text-gray-400 mb-2">Expected Parameters:</h4>
                    <div className="space-y-1">
                      {Object.entries(selectedToolInfo.params).map(([key, type]) => (
                        <div key={key} className="flex justify-between text-xs">
                          <span className="text-green-400 font-mono">{key}</span>
                          <span className="text-gray-400">{type}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* Call History */}
            <div className="flex-1 flex flex-col overflow-hidden">
              <div className="flex items-center justify-between p-4 border-b border-gray-700">
                <h3 className="text-lg font-semibold text-green-400">Call History</h3>
                <div className="flex items-center space-x-2">
                  <select
                    value={filter}
                    onChange={(e) => setFilter(e.target.value)}
                    className="px-2 py-1 bg-gray-800 border border-gray-700 rounded text-green-400 text-sm focus:border-green-500 focus:outline-none"
                  >
                    <option value="all">All</option>
                    <option value="success">Success</option>
                    <option value="error">Error</option>
                    <option value="pending">Pending</option>
                  </select>
                  <button
                    onClick={exportHistory}
                    className="p-1 text-green-400 hover:text-green-300 hover:bg-green-400/10 rounded transition-colors"
                    title="Export History"
                  >
                    <Download className="h-4 w-4" />
                  </button>
                  <button
                    onClick={clearHistory}
                    className="p-1 text-red-400 hover:text-red-300 hover:bg-red-400/10 rounded transition-colors"
                    title="Clear History"
                  >
                    <Trash2 className="h-4 w-4" />
                  </button>
                </div>
              </div>

              <div className="flex-1 overflow-auto p-4 space-y-3">
                {filteredHistory.length === 0 ? (
                  <div className="text-center py-8 text-gray-400">
                    <Terminal className="h-8 w-8 mx-auto mb-2" />
                    <p>No tool calls yet. Execute a tool to see results here.</p>
                  </div>
                ) : (
                  filteredHistory.map((call) => (
                    <div
                      key={call.id}
                      className="bg-gray-800/50 border border-gray-700 rounded-lg p-3 hover:border-green-500/40 transition-colors"
                    >
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center space-x-2">
                          {getStatusIcon(call.status)}
                          <span className="font-mono text-green-400">{call.tool}</span>
                          <span className={`px-2 py-1 rounded-full text-xs ${getStatusColor(call.status)}`}>
                            {call.status}
                          </span>
                        </div>
                        <div className="flex items-center space-x-2 text-xs text-gray-400">
                          {call.duration && <span>{call.duration}ms</span>}
                          <span>{call.timestamp.toLocaleTimeString()}</span>
                          <button
                            onClick={() => toggleResultExpansion(call.id)}
                            className="p-1 hover:bg-gray-700 rounded"
                          >
                            {expandedResults.has(call.id) ? 
                              <ChevronDown className="h-3 w-3" /> : 
                              <ChevronRight className="h-3 w-3" />
                            }
                          </button>
                        </div>
                      </div>

                      {expandedResults.has(call.id) && (
                        <div className="space-y-2 text-sm">
                          <div>
                            <div className="flex items-center justify-between mb-1">
                              <span className="text-gray-400">Arguments:</span>
                              <button
                                onClick={() => copyToClipboard(JSON.stringify(call.arguments, null, 2))}
                                className="p-1 text-gray-400 hover:text-gray-300 hover:bg-gray-700 rounded"
                                title="Copy Arguments"
                              >
                                <Copy className="h-3 w-3" />
                              </button>
                            </div>
                            <pre className="bg-gray-900 border border-gray-700 rounded p-2 text-xs overflow-auto">
                              {JSON.stringify(call.arguments, null, 2)}
                            </pre>
                          </div>

                          {call.status === 'success' && call.result && (
                            <div>
                              <div className="flex items-center justify-between mb-1">
                                <span className="text-gray-400">Result:</span>
                                <button
                                  onClick={() => copyToClipboard(JSON.stringify(call.result, null, 2))}
                                  className="p-1 text-gray-400 hover:text-gray-300 hover:bg-gray-700 rounded"
                                  title="Copy Result"
                                >
                                  <Copy className="h-3 w-3" />
                                </button>
                              </div>
                              <pre className="bg-gray-900 border border-gray-700 rounded p-2 text-xs overflow-auto text-green-400">
                                {JSON.stringify(call.result, null, 2)}
                              </pre>
                            </div>
                          )}

                          {call.status === 'error' && call.error && (
                            <div>
                              <span className="text-gray-400">Error:</span>
                              <pre className="bg-red-900/20 border border-red-500/40 rounded p-2 text-xs text-red-400 mt-1">
                                {call.error}
                              </pre>
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default McpToolsDebugger;