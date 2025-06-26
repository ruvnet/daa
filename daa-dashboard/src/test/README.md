# DAA Dashboard Test Suite

This directory contains the comprehensive test suite for the DAA Dashboard application, featuring unit tests, integration tests, and test utilities.

## Test Structure

```
src/
├── api/
│   ├── __tests__/
│   │   ├── McpClient.test.ts          # MCP client unit tests
│   │   ├── WebSocketHandler.test.ts   # WebSocket handler tests
│   │   ├── ErrorHandler.test.ts       # Error handling tests
│   │   └── DaaApiService.test.ts      # API service tests
│   └── mock/__tests__/
│       └── MockDataFactory.test.ts    # Mock data factory tests
├── components/
│   ├── __tests__/
│   │   ├── ActivityFeed.test.tsx      # Activity feed component tests
│   │   ├── AlertsPanel.test.tsx       # Alerts panel component tests
│   │   ├── MetricsChart.test.tsx      # Metrics chart component tests
│   │   └── DashboardLayout.test.tsx   # Dashboard layout tests
│   └── auth/__tests__/
│       └── LoginForm.test.tsx         # Login form component tests
├── hooks/__tests__/
│   ├── use-toast.test.tsx             # Toast hook tests
│   └── use-mobile.test.tsx            # Mobile detection hook tests
├── __tests__/
│   └── integration/
│       ├── api-integration.test.tsx   # API integration tests
│       └── real-time-updates.test.tsx # Real-time WebSocket tests
└── test/
    ├── setup.ts                       # Test setup and configuration
    ├── mocks/
    │   └── handlers.ts               # MSW request handlers
    └── utils/
        ├── test-utils.tsx            # Custom render utilities
        └── api-test-helpers.ts       # API testing helpers
```

## Running Tests

### Run all tests
```bash
npm test
```

### Run tests in watch mode
```bash
npm test -- --watch
```

### Run tests with UI
```bash
npm run test:ui
```

### Generate coverage report
```bash
npm run test:coverage
```

## Test Coverage

The test suite aims for 80%+ coverage across all components and services:

### API Layer (90%+ coverage)
- ✅ McpClient: All methods, retry logic, timeout handling
- ✅ WebSocketHandler: Connection management, event handling, reconnection
- ✅ ErrorHandler: Error classification, retry logic, React error boundary
- ✅ DaaApiService: All API methods, mock/real mode switching
- ✅ MockDataFactory: All data generation methods

### Components (85%+ coverage)
- ✅ ActivityFeed: Real-time updates, timestamp formatting, auto-refresh
- ✅ AlertsPanel: Alert management, acknowledgment, dismissal
- ✅ MetricsChart: Data visualization, chart rendering
- ✅ DashboardLayout: Navigation, responsive behavior, user interactions
- ✅ LoginForm: Form validation, submission, password visibility

### Hooks (95%+ coverage)
- ✅ useToast: Toast creation, dismissal, updates
- ✅ useMobile: Media query detection, responsive behavior

### Integration Tests
- ✅ API Integration: Component-API interaction, error handling
- ✅ Real-time Updates: WebSocket events, live data streaming
- ✅ State Management: React Query integration
- ✅ Authentication Flows: Login process, token management

## Test Utilities

### Custom Render (`test-utils.tsx`)
Provides a custom render function that includes all necessary providers:
```typescript
import { render } from '@/test/utils/test-utils'

render(<Component />, {
  route: '/dashboard',
  queryClient: customQueryClient
})
```

### API Test Helpers (`api-test-helpers.ts`)
Utilities for testing API interactions:
```typescript
const api = createMockApiService()
const { mockWebSocket, triggerMessage } = createMockWebSocket()
await waitForApiCall(mockFn)
```

### MSW Handlers (`mocks/handlers.ts`)
Mock Service Worker handlers for API endpoints:
- MCP protocol methods (initialize, tools/list, tools/call)
- Resource operations (list, read)
- All 17 DAA tools mocked with realistic responses

## Best Practices

1. **Test Isolation**: Each test should be independent and not rely on others
2. **Mock External Dependencies**: Use MSW for API calls, mock timers for time-based logic
3. **Test User Behavior**: Focus on how users interact with components
4. **Accessibility**: Use accessible queries (getByRole, getByLabelText)
5. **Async Testing**: Use waitFor for async operations, avoid arbitrary delays
6. **Error Scenarios**: Test both success and failure paths
7. **Real-time Testing**: Use act() for WebSocket event simulations

## Common Testing Patterns

### Testing Real-time Updates
```typescript
const wsHandler = (api as any).wsHandler
act(() => {
  wsHandler.emit('agent_update', { agent_id: 'test', status: 'running' })
})
await waitFor(() => {
  expect(screen.getByText('running')).toBeInTheDocument()
})
```

### Testing API Calls
```typescript
const api = createMockApiService()
const agents = await api.getAgents()
expect(agents).toHaveLength(10)
```

### Testing User Interactions
```typescript
const user = userEvent.setup()
await user.click(screen.getByRole('button', { name: 'Submit' }))
await user.type(screen.getByLabelText('Email'), 'test@example.com')
```

## Debugging Tips

1. Use `screen.debug()` to see the current DOM
2. Use `screen.logTestingPlaygroundURL()` for interactive debugging
3. Set `DEBUG_PRINT_LIMIT=0` for unlimited debug output
4. Use `vi.spyOn` to monitor function calls
5. Check MSW logs for unhandled requests

## Future Improvements

- [ ] Add visual regression tests with Playwright
- [ ] Implement performance benchmarks
- [ ] Add mutation testing with Stryker
- [ ] Create component storybook tests
- [ ] Add E2E tests for critical user flows