# DAA Global Business Dashboard

A comprehensive Next.js 14 web-based management platform for operating, monitoring, and scaling Decentralized Autonomous Agent (DAA) infrastructure at enterprise scale.

## 🚀 Quick Start

### Prerequisites

- Node.js 18.0.0 or higher
- npm, yarn, pnpm, or bun package manager
- Git

### Installation

1. Clone the repository:
```bash
git clone https://github.com/your-org/daa-dashboard.git
cd daa-dashboard
```

2. Install dependencies:
```bash
npm install
# or
yarn install
# or
pnpm install
# or
bun install
```

3. Set up environment variables:
```bash
cp .env.example .env.local
```

4. Edit `.env.local` with your configuration:
```env
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080
NEXT_PUBLIC_MCP_URL=http://localhost:3000
# Add other required environment variables
```

5. Run the development server:
```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the dashboard.

## 📁 Project Structure

```
daa-dashboard/
├── src/
│   ├── app/                      # Next.js App Router
│   │   ├── (auth)/              # Auth layout group
│   │   ├── (dashboard)/         # Dashboard layout group
│   │   ├── api/                 # API routes
│   │   └── layout.tsx           # Root layout
│   ├── components/              # React components
│   │   ├── ui/                 # Base UI components (Shadcn/ui)
│   │   ├── dashboard/          # Dashboard-specific components
│   │   └── ...                 # Feature components
│   ├── hooks/                  # Custom React hooks
│   ├── lib/                    # Core libraries
│   │   ├── api/               # API clients
│   │   ├── auth/              # Auth utilities
│   │   └── utils/             # Utility functions
│   ├── stores/                # Zustand stores
│   ├── types/                 # TypeScript types
│   ├── services/              # Business logic
│   └── middleware/            # Next.js middleware
├── tests/                     # Test files
├── public/                    # Static assets
└── ...config files
```

## 🛠️ Technology Stack

### Core Technologies
- **Framework**: Next.js 14 (App Router)
- **Language**: TypeScript 5.x
- **Styling**: Tailwind CSS + Shadcn/ui components
- **State Management**: Zustand + React Query (TanStack Query)
- **Real-time**: Socket.io-client + Server-Sent Events
- **Charts**: Chart.js + Recharts
- **Maps**: Mapbox GL JS

### Development Tools
- **Build Tool**: Next.js built-in
- **Testing**: Vitest + React Testing Library + Playwright
- **Linting**: ESLint + Prettier
- **Version Control**: Git with Husky for pre-commit hooks

## 📜 Available Scripts

### Development
```bash
npm run dev          # Start development server
npm run build        # Build for production
npm run start        # Start production server
```

### Code Quality
```bash
npm run lint         # Run ESLint
npm run lint:fix     # Fix ESLint errors
npm run format       # Format code with Prettier
npm run format:check # Check formatting
npm run typecheck    # Run TypeScript type checking
```

### Testing
```bash
npm run test         # Run unit tests
npm run test:ui      # Run tests with UI
npm run test:coverage # Generate coverage report
npm run test:e2e     # Run E2E tests with Playwright
npm run test:e2e:ui  # Run E2E tests with UI
```

## 🎨 Design System

### Color Palette

The dashboard uses a comprehensive color system defined in `tailwind.config.ts`:

- **Primary**: `#2563eb` (DAA Blue)
- **Secondary**: `#059669` (Success Green)
- **Warning**: `#d97706` (Warning Orange)
- **Error**: `#dc2626` (Error Red)
- **Info**: `#0891b2` (Info Cyan)

### Typography

- **Sans**: Inter, system-ui
- **Mono**: JetBrains Mono, Consolas

## 🔧 Configuration

### Environment Variables

See `.env.example` for all available environment variables:

- `NEXT_PUBLIC_API_URL`: Backend API URL
- `NEXT_PUBLIC_WS_URL`: WebSocket server URL
- `NEXT_PUBLIC_MCP_URL`: MCP server URL
- `NEXT_PUBLIC_MAPBOX_TOKEN`: Mapbox access token
- And more...

### TypeScript Configuration

The project uses strict TypeScript settings. Configuration can be found in:
- `tsconfig.json` - Main TypeScript config
- Path aliases are configured for clean imports (`@/components`, `@/lib`, etc.)

### ESLint Configuration

ESLint is configured with:
- Next.js recommended rules
- TypeScript ESLint
- React hooks rules
- Prettier integration

## 🧪 Testing

### Unit Tests
```bash
npm run test
```

Unit tests are written with Vitest and React Testing Library. Test files should be placed next to the components they test with a `.test.tsx` extension.

### Integration Tests
```bash
npm run test:integration
```

Integration tests test the interaction between multiple components and services.

### E2E Tests
```bash
npm run test:e2e
```

E2E tests use Playwright to test complete user workflows.

## 📦 Building for Production

1. Build the application:
```bash
npm run build
```

2. Start the production server:
```bash
npm run start
```

### Docker Deployment

```dockerfile
# See Dockerfile in repository root
docker build -t daa-dashboard .
docker run -p 3000:3000 daa-dashboard
```

## 🔐 Security

- Authentication via NextAuth.js
- Role-based access control (RBAC)
- API security with JWT tokens
- Environment variables for sensitive data
- Content Security Policy headers

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Convention

Follow conventional commits:
- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `style:` Code style changes
- `refactor:` Code refactoring
- `test:` Test additions/changes
- `chore:` Build process or auxiliary tool changes

## 📚 Documentation

- [Implementation Plan](./IMPLEMENTATION_PLAN.md)
- [API Integration Design](./API_INTEGRATION_DESIGN.md)
- [UI Dashboard Specification](/workspaces/daa/plans/ui-dashboard.md)

## 🐛 Troubleshooting

### Common Issues

1. **Module not found errors**: Clear node_modules and reinstall:
```bash
rm -rf node_modules package-lock.json
npm install
```

2. **Type errors**: Run type checking:
```bash
npm run typecheck
```

3. **Build failures**: Check environment variables and ensure all required services are running.

## 📄 License

This project is proprietary software. All rights reserved.

## 👥 Team

- DAA Development Team

## 🔗 Links

- [DAA Documentation](https://docs.daa.network)
- [API Reference](https://api.daa.network/docs)
- [Support](https://support.daa.network)