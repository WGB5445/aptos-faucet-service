import React, { useEffect, useState, useRef } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { googleAuth } from '../lib/googleAuth';

const TestLoginPage: React.FC = () => {
  const { user, isAuthenticated, signIn, signOut, refreshUser } = useAuth();
  const [clientId, setClientId] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [sessionInfo, setSessionInfo] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(false);
  const googleButtonRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // 获取环境变量中的客户端ID
    const envClientId = (import.meta as any).env?.VITE_GOOGLE_CLIENT_ID || '';
    setClientId(envClientId);
    
    if (!envClientId) {
      setError('未找到 VITE_GOOGLE_CLIENT_ID 环境变量');
    }

    // 获取session信息
    const token = localStorage.getItem('auth_token');
    setSessionInfo({
      hasToken: !!token,
      tokenLength: token?.length || 0,
      isAuthenticated,
      user: user ? {
        handle: user.handle,
        role: user.role,
        max_amount: user.max_amount
      } : null
    });
  }, [isAuthenticated, user]);

  // 初始化Google Auth按钮
  useEffect(() => {
    const initGoogleAuth = async () => {
      try {
        await googleAuth.initialize();
        if (googleButtonRef.current && !isAuthenticated) {
          googleAuth.renderButton(googleButtonRef.current.id);
        }
      } catch (err) {
        setError('Google 登录初始化失败');
      }
    };

    if (!isAuthenticated) {
      initGoogleAuth();
    }
  }, [isAuthenticated]);

  const handleGoogleSignIn = async () => {
    try {
      setIsLoading(true);
      setError('');
      await signIn();
    } catch (err) {
      setError('登录失败: ' + (err as Error).message);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSignOut = async () => {
    try {
      await signOut();
      setError('');
    } catch (err) {
      setError('登出失败: ' + (err as Error).message);
    }
  };

  const handleRefreshUser = async () => {
    try {
      await refreshUser();
      setError('');
    } catch (err) {
      setError('刷新用户信息失败: ' + (err as Error).message);
    }
  };

  const handleCheckJWT = () => {
    const token = localStorage.getItem('auth_token');
    
    if (token) {
      try {
        // 解析JWT payload
        const payload = JSON.parse(atob(token.split('.')[1]));
        const isExpired = Date.now() > payload.exp * 1000;
        setError(`JWT检查完成 - 过期状态: ${isExpired ? '已过期' : '有效'}`);
      } catch (e) {
        setError('JWT解析失败');
      }
    } else {
      setError('没有找到JWT token');
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-2xl w-full space-y-8">
        <div className="text-center">
          <h2 className="text-3xl font-extrabold text-gray-900">
            Session 管理测试页面
          </h2>
          <p className="mt-2 text-sm text-gray-600">
            用于测试 Session 自动刷新和状态管理
          </p>
        </div>

        <div className="mt-8 space-y-6">
          {error && (
            <div className="bg-red-50 border border-red-200 rounded-md p-4">
              <p className="text-sm text-red-800">{error}</p>
            </div>
          )}

          {/* Session 状态信息 */}
          <div className="bg-gray-50 p-4 rounded-md">
            <h3 className="text-sm font-medium text-gray-900 mb-2">Session 状态</h3>
            <div className="space-y-2">
              <p className="text-sm text-gray-600">
                登录状态: <span className={isAuthenticated ? 'text-green-600' : 'text-red-600'}>
                  {isAuthenticated ? '已登录' : '未登录'}
                </span>
              </p>
              <p className="text-sm text-gray-600">
                本地Token: <span className={sessionInfo?.hasToken ? 'text-green-600' : 'text-red-600'}>
                  {sessionInfo?.hasToken ? `存在 (${sessionInfo.tokenLength} 字符)` : '不存在'}
                </span>
              </p>
              <p className="text-sm text-gray-600">
                加载状态: <span className={isLoading ? 'text-yellow-600' : 'text-green-600'}>
                  {isLoading ? '加载中...' : '已完成'}
                </span>
              </p>
              {user && (
                <div className="mt-2 p-2 bg-white rounded border">
                  <p className="text-sm font-medium text-gray-900">用户信息:</p>
                  <p className="text-sm text-gray-600">Handle: {user.handle}</p>
                  <p className="text-sm text-gray-600">角色: {user.role}</p>
                  <p className="text-sm text-gray-600">最大金额: {user.max_amount}</p>
                </div>
              )}
              {/* 调试信息 */}
              <div className="mt-2 p-2 bg-blue-50 rounded border">
                <p className="text-sm font-medium text-blue-900">调试信息:</p>
                <p className="text-xs text-blue-700">localStorage token: {localStorage.getItem('auth_token') ? '存在' : '不存在'}</p>
                <p className="text-xs text-blue-700">API Base URL: {(import.meta as any).env?.VITE_API_BASE_URL || '/api'}</p>
              </div>
            </div>
          </div>

          {/* 配置信息 */}
          <div className="bg-gray-50 p-4 rounded-md">
            <h3 className="text-sm font-medium text-gray-900 mb-2">当前配置</h3>
            <p className="text-sm text-gray-600">
              Client ID: {clientId ? `${clientId.substring(0, 20)}...` : '未配置'}
            </p>
            <p className="text-sm text-gray-600">
              当前域名: {window.location.origin}
            </p>
          </div>

          {/* 测试按钮 */}
          <div className="space-y-3">
            {!isAuthenticated ? (
              <div className="space-y-3">
                {/* Google 登录按钮容器 */}
                <div 
                  ref={googleButtonRef}
                  id="google-signin-button"
                  className="w-full"
                ></div>
                
                {/* 备用登录按钮 */}
                <button
                  onClick={handleGoogleSignIn}
                  disabled={isLoading}
                  className="w-full flex justify-center py-3 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isLoading ? (
                    <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white"></div>
                  ) : (
                    '测试 Google 登录 (备用)'
                  )}
                </button>
              </div>
            ) : (
              <div className="space-y-2">
                <button
                  onClick={handleSignOut}
                  className="w-full flex justify-center py-3 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
                >
                  登出
                </button>
                <button
                  onClick={handleRefreshUser}
                  className="w-full flex justify-center py-3 px-4 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                >
                  刷新用户信息
                </button>
                <button
                  onClick={handleCheckJWT}
                  className="w-full flex justify-center py-3 px-4 border border-blue-300 text-sm font-medium rounded-md text-blue-700 bg-blue-50 hover:bg-blue-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                >
                  检查JWT Token
                </button>
              </div>
            )}
          </div>

          {/* JWT自动登录说明 */}
          <div className="bg-green-50 p-4 rounded-md">
            <h3 className="text-sm font-medium text-green-900 mb-2">JWT自动登录</h3>
            <ul className="text-sm text-green-800 space-y-1">
              <li>• 首次Google登录后，JWT自动保存到localStorage</li>
              <li>• 页面刷新时自动检查JWT并恢复登录状态</li>
              <li>• 无需重复Google登录，直接使用JWT验证</li>
              <li>• JWT过期后会自动清除并提示重新登录</li>
            </ul>
          </div>

          {/* 测试说明 */}
          <div className="bg-yellow-50 p-4 rounded-md">
            <h3 className="text-sm font-medium text-yellow-900 mb-2">测试JWT自动登录</h3>
            <ol className="text-sm text-yellow-800 space-y-1">
              <li>1. 先点击Google登录完成首次登录</li>
              <li>2. 刷新页面，应该自动恢复登录状态</li>
              <li>3. 关闭浏览器重新打开，应该仍然保持登录</li>
              <li>4. 检查localStorage中是否有auth_token</li>
            </ol>
          </div>

          <div className="text-center">
            <p className="text-xs text-gray-500">
              测试完成后，可以刷新页面验证session是否保持
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default TestLoginPage;
