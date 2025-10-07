import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { User } from '../types';
import { authApi } from '../lib/api';
import { googleAuth, GoogleUser } from '../lib/googleAuth';

interface AuthContextType {
  user: User | null;
  googleUser: GoogleUser | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  signIn: () => Promise<void>;
  signOut: () => Promise<void>;
  refreshUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

interface AuthProviderProps {
  children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [googleUser, setGoogleUser] = useState<GoogleUser | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const isAuthenticated = !!user;

  const refreshUser = async (): Promise<void> => {
    try {
      const userData = await authApi.getCurrentUser();
      setUser(userData);
    } catch (error) {
      // 如果token过期或无效，清除本地存储
      localStorage.removeItem('auth_token');
      setUser(null);
      setGoogleUser(null);
    }
  };

  const signIn = async () => {
    try {
      setIsLoading(true);
      const { user: googleUserData, session } = await googleAuth.signIn();
      setGoogleUser(googleUserData);
      setUser(session.user);
    } catch (error) {
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  const signOut = async () => {
    try {
      await googleAuth.signOut();
    } catch (error) {
      // 忽略登出错误
    } finally {
      // 清除本地存储的token
      localStorage.removeItem('auth_token');
      setUser(null);
      setGoogleUser(null);
    }
  };

  useEffect(() => {
    const initializeAuth = async () => {
      try {
        // Check if we have a stored token
        const token = localStorage.getItem('auth_token');
        
        if (token) {
          // Try to get current user
          await refreshUser();
        }
      } catch (error) {
        // Clear invalid token
        localStorage.removeItem('auth_token');
      } finally {
        setIsLoading(false);
      }
    };

    // 监听Google登录成功事件
    const handleGoogleSignIn = (event: any) => {
      const { user: googleUserData, session } = event.detail;
      setGoogleUser(googleUserData);
      setUser(session.user);
      setIsLoading(false);
    };

    // 监听Google登录失败事件
    const handleGoogleSignInError = () => {
      setIsLoading(false);
    };

    window.addEventListener('googleSignIn', handleGoogleSignIn);
    window.addEventListener('googleSignInError', handleGoogleSignInError);

    initializeAuth();

    return () => {
      window.removeEventListener('googleSignIn', handleGoogleSignIn);
      window.removeEventListener('googleSignInError', handleGoogleSignInError);
    };
  }, []);

  const value: AuthContextType = {
    user,
    googleUser,
    isLoading,
    isAuthenticated,
    signIn,
    signOut,
    refreshUser,
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
};
