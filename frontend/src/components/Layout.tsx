import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { Droplets, LogOut, Shield } from 'lucide-react';

interface LayoutProps {
  children: React.ReactNode;
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
  const { user, signOut, googleUser } = useAuth();
  const location = useLocation();

  const isAdmin = user?.role === 'admin';

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            {/* Logo */}
            <div className="flex items-center">
              <Link to="/" className="flex items-center space-x-2">
                <Droplets className="h-8 w-8 text-primary-600" />
                <span className="text-xl font-bold text-gray-900">水龙头服务</span>
              </Link>
            </div>

            {/* Navigation */}
            <nav className="hidden md:flex space-x-8">
              <Link
                to="/"
                className={`px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                  location.pathname === '/'
                    ? 'text-primary-600 bg-primary-50'
                    : 'text-gray-500 hover:text-gray-700 hover:bg-gray-50'
                }`}
              >
                首页
              </Link>
              {isAdmin && (
                <Link
                  to="/admin"
                  className={`px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                    location.pathname === '/admin'
                      ? 'text-primary-600 bg-primary-50'
                      : 'text-gray-500 hover:text-gray-700 hover:bg-gray-50'
                  }`}
                >
                  管理后台
                </Link>
              )}
            </nav>

            {/* User menu */}
            <div className="flex items-center space-x-4">
              {/* User info */}
              <div className="flex items-center space-x-3">
                {googleUser?.picture && (
                  <img
                    src={googleUser.picture}
                    alt={googleUser.name}
                    className="h-8 w-8 rounded-full"
                  />
                )}
                <div className="hidden sm:block">
                  <div className="flex items-center space-x-1">
                    <span className="text-sm font-medium text-gray-900">
                      {googleUser?.name || user?.handle}
                    </span>
                    {isAdmin && (
                      <Shield className="h-4 w-4 text-yellow-500" />
                    )}
                  </div>
                  <div className="text-xs text-gray-500 capitalize">
                    {user?.role}
                  </div>
                </div>
              </div>

              {/* Sign out button */}
              <button
                onClick={signOut}
                className="btn btn-outline btn-sm"
              >
                <LogOut className="h-4 w-4 mr-1" />
                退出
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1">
        {children}
      </main>

      {/* Footer */}
      <footer className="bg-white border-t">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="text-center text-sm text-gray-500">
            <p>&copy; 2024 水龙头服务. 现代化的代币分发平台</p>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Layout;
