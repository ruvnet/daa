
import React, { useState } from 'react';
import { Mail, ArrowLeft, Terminal, Check } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

interface ForgotPasswordFormProps {
  onSwitchToLogin: () => void;
  onResetPassword: (email: string) => void;
}

const ForgotPasswordForm = ({ onSwitchToLogin, onResetPassword }: ForgotPasswordFormProps) => {
  const [email, setEmail] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isSuccess, setIsSuccess] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    
    // Simulate API call
    setTimeout(() => {
      onResetPassword(email);
      setIsSuccess(true);
      setIsLoading(false);
    }, 1000);
  };

  if (isSuccess) {
    return (
      <div className="min-h-screen bg-black flex items-center justify-center p-4">
        <div className="w-full max-w-md">
          <div className="text-center mb-8">
            <div className="flex items-center justify-center mb-4">
              <div className="w-12 h-12 bg-green-500 rounded-lg flex items-center justify-center">
                <Check className="h-6 w-6 text-black" />
              </div>
            </div>
            <h1 className="text-2xl font-bold text-green-400 font-mono mb-2">Reset Link Sent</h1>
            <p className="text-green-400/70 text-sm font-mono">Check your email for instructions</p>
          </div>

          <div className="bg-gray-900/50 border border-green-500/20 rounded-lg p-6 text-center">
            <Mail className="h-12 w-12 text-green-400 mx-auto mb-4" />
            <p className="text-green-400 font-mono text-sm mb-4">
              We've sent a password reset link to:
            </p>
            <p className="text-green-300 font-mono font-bold mb-6">{email}</p>
            <p className="text-green-400/70 font-mono text-xs mb-6">
              If you don't see the email, check your spam folder or try again.
            </p>
            
            <Button
              onClick={onSwitchToLogin}
              className="w-full bg-green-500 hover:bg-green-600 text-black font-mono font-bold"
            >
              <div className="flex items-center space-x-2">
                <ArrowLeft className="h-4 w-4" />
                <span>BACK TO LOGIN</span>
              </div>
            </Button>
          </div>

          <div className="mt-6 text-center">
            <p className="text-green-400/50 font-mono text-xs">
              © 2025 DAA Technologies • Quantum-Secured Platform
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-black flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="flex items-center justify-center mb-4">
            <div className="w-12 h-12 bg-green-500 rounded-lg flex items-center justify-center">
              <Terminal className="h-6 w-6 text-black" />
            </div>
          </div>
          <h1 className="text-2xl font-bold text-green-400 font-mono mb-2">Reset Password</h1>
          <p className="text-green-400/70 text-sm font-mono">Enter your email to reset password</p>
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
                placeholder="admin@daa.global"
                required
              />
            </div>

            <Button
              type="submit"
              disabled={isLoading}
              className="w-full bg-green-500 hover:bg-green-600 text-black font-mono font-bold"
            >
              {isLoading ? (
                <div className="flex items-center space-x-2">
                  <div className="w-4 h-4 border-2 border-black/30 border-t-black rounded-full animate-spin"></div>
                  <span>SENDING RESET LINK...</span>
                </div>
              ) : (
                <div className="flex items-center space-x-2">
                  <Mail className="h-4 w-4" />
                  <span>SEND RESET LINK</span>
                </div>
              )}
            </Button>
          </form>

          <div className="mt-6 text-center">
            <button
              onClick={onSwitchToLogin}
              className="text-green-400/70 hover:text-green-400 font-mono text-sm flex items-center justify-center space-x-2"
            >
              <ArrowLeft className="h-4 w-4" />
              <span>Back to Login</span>
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

export default ForgotPasswordForm;
