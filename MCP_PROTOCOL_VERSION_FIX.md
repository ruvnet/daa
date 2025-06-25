# MCP Protocol Version Fix

## Issue
Claude Code was rejecting the MCP server with the error:
```
Server's protocol version is not supported: 1.0.0
```

## Root Cause Analysis

### Research Findings:
1. **Protocol Version Format**: MCP uses date-based versioning (e.g., "2024-11-05"), not semantic versioning (e.g., "1.0.0")
2. **Current Support**: Claude Code supports the older "2024-11-05" protocol version
3. **Version 2025-03-26**: The newer protocol version is not yet supported by most MCP clients including Claude Code

### Evidence from Codebase:
- DAA MCP defines: `MCP_PROTOCOL_VERSION: "2025-03-26"` (too new)
- QuDAG MCP defines: `MCP_PROTOCOL_VERSION: "2025-03-26"` (too new)
- Both would fail with current Claude Code

## Solution Applied

### 1. Corrected Protocol Version
Changed from `"1.0.0"` to `"2024-11-05"` (stable, supported version)

**Files Updated:**
- `/workspaces/daa/daa-mcp-server.py` - Line 48
- `/workspaces/daa/.roo/mcp.json` - Line 25

### 2. Protocol Version Compatibility Matrix

| Version | Format | Claude Code Support | Status |
|---------|--------|-------------------|---------|
| `1.0.0` | Semantic | ❌ Invalid | Incorrect format |
| `2024-11-05` | Date | ✅ Supported | **Recommended** |
| `2025-03-26` | Date | ❌ Not yet | Too new |

### 3. Testing Results
```bash
# Before fix:
# Error: Server's protocol version is not supported: 1.0.0

# After fix:
{"jsonrpc":"2.0","result":{"protocolVersion":"2024-11-05",...}}
```

## Future Considerations

### When to Upgrade Protocol Version:
1. **Wait for Claude Code support** of newer protocol versions
2. **Monitor MCP specification updates** and client compatibility
3. **Test thoroughly** before upgrading to avoid breaking changes

### Version Migration Strategy:
1. Keep using `2024-11-05` for maximum compatibility
2. When Claude Code supports `2025-03-26`, update all servers simultaneously
3. Consider feature detection for version-specific capabilities

## References
- [MCP Specification Changes](https://modelcontextprotocol.io/quickstart/user)
- [Claude Code MCP Issues](https://github.com/anthropics/claude-code/issues/768)
- DAA MCP implementation: `/workspaces/daa/daa-mcp/src/lib.rs:68`
- QuDAG MCP implementation: `/workspaces/daa/qudag/qudag-mcp/src/lib.rs:73`

## Action Items
- [x] Fix protocol version in DAA MCP server
- [x] Update configuration files
- [x] Test server functionality
- [ ] Consider updating DAA/QuDAG MCP libraries to use compatible version
- [ ] Monitor Claude Code updates for newer protocol support