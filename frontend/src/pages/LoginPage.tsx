import React, { useState, useEffect, useRef } from 'react';
import { Droplets, AlertCircle } from 'lucide-react';
import { googleAuth } from '../lib/googleAuth';

const LoginPage: React.FC = () => {
  const [error, setError] = useState<string | null>(null);
  const googleButtonRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // 初始化 Google Auth 并渲染按钮
    const initGoogleAuth = async () => {
      try {
        await googleAuth.initialize();
        if (googleButtonRef.current) {
          googleAuth.renderButton(googleButtonRef.current.id);
        }
      } catch (err) {
        console.error('Failed to initialize Google Auth:', err);
        setError('Google 登录初始化失败');
      }
    };

    initGoogleAuth();
  }, []);


  return (
    <div className="min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        <div className="text-center">
          <div className="mx-auto h-16 w-16 bg-primary-100 rounded-full flex items-center justify-center">
            <Droplets className="h-8 w-8 text-primary-600" />
          </div>
          <h2 className="mt-6 text-3xl font-extrabold text-gray-900">
            欢迎使用水龙头服务
          </h2>
          <p className="mt-2 text-sm text-gray-600">
            请使用 Google 账户登录以开始领取代币
          </p>
        </div>

        <div className="mt-8 space-y-6">
          {error && (
            <div className="bg-red-50 border border-red-200 rounded-md p-4">
              <div className="flex">
                <AlertCircle className="h-5 w-5 text-red-400" />
                <div className="ml-3">
                  <p className="text-sm text-red-800">{error}</p>
                </div>
              </div>
            </div>
          )}

          <div>
            {/* Google 登录按钮容器 */}
            <div 
              ref={googleButtonRef}
              id="google-signin-button"
              className="w-full"
            ></div>
          </div>

          <div className="text-center">
            <p className="text-xs text-gray-500">
              登录即表示您同意我们的服务条款和隐私政策
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LoginPage;
