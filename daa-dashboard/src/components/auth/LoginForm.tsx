import React, { useState } from 'react';
import { Eye, EyeOff, Shield, Terminal } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

interface LoginFormProps {
  onSwitchToRegister: () => void;
  onSwitchToForgotPassword: () => void;
  onLogin: (email: string, password: string) => void;
}

const LoginForm = ({ onSwitchToRegister, onSwitchToForgotPassword, onLogin }: LoginFormProps) => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    
    // Simulate API call
    setTimeout(() => {
      onLogin(email, password);
      setIsLoading(false);
    }, 1000);
  };

  return (
    <div className="min-h-screen bg-black flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="flex items-center justify-center mb-4">
            <div className="w-12 h-12 bg-green-500 rounded-lg flex items-center justify-center">
              <Terminal className="h-6 w-6 text-black" />
            </div>
          </div>
          <h1 className="text-2xl font-bold text-green-400 font-mono mb-2">DAA Global Command</h1>
          <p className="text-green-400/70 text-sm font-mono">Secure Access Portal</p>
        </div>

        <div className="bg-gray-900/50 border border-green-500/20 rounded-lg p-6">
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <Label htmlFor="email" className="text-green-400 font-mono text-sm">
                Email Address
              </Label>
              <Input
                id="email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="bg-black/50 border-green-500/30 text-green-400 font-mono focus:border-green-500"
                placeholder="demo@daa.dark"
                required
              />
            </div>

            <div>
              <Label htmlFor="password" className="text-green-400 font-mono text-sm">
                Password
              </Label>
              <div className="relative">
                <Input
                  id="password"
                  type={showPassword ? 'text' : 'password'}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="bg-black/50 border-green-500/30 text-green-400 font-mono focus:border-green-500 pr-10"
                  placeholder="••••••••"
                  required
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-green-400/70 hover:text-green-400"
                >
                  {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
            </div>

            <Button
              type="submit"
              disabled={isLoading}
              className="w-full bg-green-500 hover:bg-green-600 text-black font-mono font-bold"
            >
              {isLoading ? (
                <div className="flex items-center space-x-2">
                  <div className="w-4 h-4 border-2 border-black/30 border-t-black rounded-full animate-spin"></div>
                  <span>AUTHENTICATING...</span>
                </div>
              ) : (
                <div className="flex items-center space-x-2">
                  <Shield className="h-4 w-4" />
                  <span>ACCESS SYSTEM</span>
                </div>
              )}
            </Button>
          </form>

          <div className="mt-6 space-y-3 text-center">
            <button
              onClick={onSwitchToForgotPassword}
              className="text-green-400/70 hover:text-green-400 font-mono text-sm"
            >
              Forgot Password?
            </button>
            <div className="text-green-400/50 font-mono text-xs">
              ──────── OR ────────
            </div>
            <button
              onClick={onSwitchToRegister}
              className="text-green-400 hover:text-green-300 font-mono text-sm"
            >
              Create New Account
            </button>
          </div>
        </div>

        <div className="mt-6 text-center">
          <p className="text-green-400/50 font-mono text-xs">
            © 2025 DAA Technologies • Quantum-Secured Platform
          </p>
        </div>
      </div>
    </div>
  );
};

export default LoginForm;
