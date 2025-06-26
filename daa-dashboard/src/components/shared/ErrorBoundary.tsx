import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
    errorInfo: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error, errorInfo: null };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    this.setState({
      error,
      errorInfo,
    });
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null, errorInfo: null });
    window.location.reload();
  };

  private handleGoHome = () => {
    this.setState({ hasError: false, error: null, errorInfo: null });
    window.location.href = '/';
  };

  public render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return <>{this.props.fallback}</>;
      }

      return (
        <div className="min-h-screen bg-black text-green-400 font-mono flex items-center justify-center p-4">
          <Card className="bg-gray-900/50 border-red-500/20 max-w-2xl w-full">
            <CardHeader className="pb-4">
              <CardTitle className="text-red-400 flex items-center text-xl">
                <AlertTriangle className="h-6 w-6 mr-2" />
                System Error Detected
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="bg-black/50 border border-red-500/20 rounded-lg p-4">
                <p className="text-red-400/80 text-sm mb-2">ERROR_TYPE: RUNTIME_EXCEPTION</p>
                <p className="text-green-400/70 text-xs break-all">
                  {this.state.error?.message || 'An unexpected error occurred'}
                </p>
              </div>

              {process.env.NODE_ENV === 'development' && this.state.errorInfo && (
                <div className="bg-black/50 border border-red-500/20 rounded-lg p-4">
                  <p className="text-red-400/80 text-sm mb-2">STACK_TRACE:</p>
                  <pre className="text-green-400/50 text-xs overflow-auto max-h-48">
                    {this.state.error?.stack}
                  </pre>
                  <p className="text-red-400/80 text-sm mt-4 mb-2">COMPONENT_STACK:</p>
                  <pre className="text-green-400/50 text-xs overflow-auto max-h-48">
                    {this.state.errorInfo.componentStack}
                  </pre>
                </div>
              )}

              <div className="flex flex-col sm:flex-row gap-3 pt-4">
                <Button
                  onClick={this.handleReset}
                  className="bg-green-500 hover:bg-green-600 text-black font-mono flex items-center justify-center"
                >
                  <RefreshCw className="h-4 w-4 mr-2" />
                  Restart System
                </Button>
                <Button
                  onClick={this.handleGoHome}
                  variant="outline"
                  className="border-green-500/30 text-green-400 hover:bg-green-500/10 font-mono flex items-center justify-center"
                >
                  <Home className="h-4 w-4 mr-2" />
                  Return to Dashboard
                </Button>
              </div>

              <div className="text-green-400/50 text-xs pt-4">
                <p>INCIDENT_ID: {Date.now()}</p>
                <p>TIMESTAMP: {new Date().toISOString()}</p>
                <p>Please contact system administrator if issue persists.</p>
              </div>
            </CardContent>
          </Card>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;