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

  const refreshUser = async () => {
    try {
      console.log('AuthContext: Refreshing user...');
      const token = localStorage.getItem('auth_token');
      console.log('AuthContext: Current token:', token ? `${token.substring(0, 20)}...` : 'null');
      const userData = await authApi.getCurrentUser();
      console.log('AuthContext: User data received:', userData);
      setUser(userData);
      console.log('AuthContext: User state updated successfully');
      return true;
    } catch (error) {
      console.error('AuthContext: Failed to refresh user:', error);
      console.error('AuthContext: Error details:', error);
      // 如果token过期或无效，清除本地存储
      localStorage.removeItem('auth_token');
      setUser(null);
      setGoogleUser(null);
      return false;
    }
  };

  const signIn = async () => {
    try {
      setIsLoading(true);
      console.log('AuthContext: Starting Google sign in...');
      const { user: googleUserData, session } = await googleAuth.signIn();
      console.log('AuthContext: Google sign in successful:', { googleUserData, session });
      console.log('AuthContext: Setting user state...');
      setGoogleUser(googleUserData);
      setUser(session.user);
      console.log('AuthContext: User state updated');
    } catch (error) {
      console.error('AuthContext: Sign in failed:', error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  const signOut = async () => {
    try {
      await googleAuth.signOut();
    } catch (error) {
      console.error('Google sign out failed:', error);
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
        console.log('AuthContext: Initializing auth...');
        // Check if we have a stored token
        const token = localStorage.getItem('auth_token');
        console.log('AuthContext: Stored token:', token ? 'exists' : 'not found');
        console.log('AuthContext: Token preview:', token ? `${token.substring(0, 50)}...` : 'null');
        console.log('AuthContext: Token length:', token?.length || 0);
        
        if (token) {
          console.log('AuthContext: Found token, attempting to refresh user...');
          // Try to get current user
          const success = await refreshUser();
          console.log('AuthContext: Refresh user result:', success);
        } else {
          console.log('AuthContext: No token found, user remains unauthenticated');
        }
      } catch (error) {
        console.error('AuthContext: Initialization failed:', error);
        console.error('AuthContext: Error details:', error);
        // Clear invalid token
        localStorage.removeItem('auth_token');
        console.log('AuthContext: Cleared invalid token from localStorage');
      } finally {
        console.log('AuthContext: Setting isLoading to false');
        setIsLoading(false);
      }
    };

    // 监听Google登录成功事件
    const handleGoogleSignIn = (event: any) => {
      console.log('AuthContext: Received googleSignIn event:', event.detail);
      const { user: googleUserData, session } = event.detail;
      setGoogleUser(googleUserData);
      setUser(session.user);
      setIsLoading(false);
    };

    // 监听Google登录失败事件
    const handleGoogleSignInError = (event: any) => {
      console.log('AuthContext: Received googleSignInError event:', event.detail);
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
