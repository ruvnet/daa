
import React, { useState } from 'react';
import LoginForm from '@/components/auth/LoginForm';
import RegisterForm from '@/components/auth/RegisterForm';
import ForgotPasswordForm from '@/components/auth/ForgotPasswordForm';

type AuthMode = 'login' | 'register' | 'forgot-password';

interface AuthProps {
  onAuthenticated: () => void;
}

const Auth = ({ onAuthenticated }: AuthProps) => {
  const [mode, setMode] = useState<AuthMode>('login');

  const handleLogin = (email: string, password: string) => {
    console.log('Login:', { email, password });
    // Here you would typically make an API call
    onAuthenticated();
  };

  const handleRegister = (data: { email: string; password: string; firstName: string; lastName: string; organization: string }) => {
    console.log('Register:', data);
    // Here you would typically make an API call
    onAuthenticated();
  };

  const handleResetPassword = (email: string) => {
    console.log('Reset password for:', email);
    // Here you would typically make an API call
  };

  switch (mode) {
    case 'login':
      return (
        <LoginForm
          onSwitchToRegister={() => setMode('register')}
          onSwitchToForgotPassword={() => setMode('forgot-password')}
          onLogin={handleLogin}
        />
      );
    case 'register':
      return (
        <RegisterForm
          onSwitchToLogin={() => setMode('login')}
          onRegister={handleRegister}
        />
      );
    case 'forgot-password':
      return (
        <ForgotPasswordForm
          onSwitchToLogin={() => setMode('login')}
          onResetPassword={handleResetPassword}
        />
      );
    default:
      return null;
  }
};

export default Auth;
