import React, { useState, useEffect, useRef } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { faucetApi } from '../lib/api';
import { MintResponse } from '../types';
import { 
  Droplets, 
  CheckCircle, 
  XCircle, 
  Clock, 
  AlertCircle,
  RefreshCw,
  Coins,
  Zap,
  LogIn,
  User,
  Shield,
  Star,
  Wallet
} from 'lucide-react';
import { googleAuth } from '../lib/googleAuth';

const LandingPage: React.FC = () => {
  const { user, isAuthenticated, signOut, refreshUser } = useAuth();
  const [isMinting, setIsMinting] = useState(false);
  const [mintResult, setMintResult] = useState<MintResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [customAmount, setCustomAmount] = useState<string>('');
  const [walletAddress, setWalletAddress] = useState<string>('');
  const googleButtonRef = useRef<HTMLDivElement>(null);

  // 处理URL参数中的address
  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const address = urlParams.get('address');
    if (address) {
      setWalletAddress(address);
    }
  }, []);

  const formatAmount = (amount: number) => {
    return (amount / 1e8).toFixed(2);
  };

  const handleMint = async (amount?: number) => {
    if (!isAuthenticated) {
      setError('请先登录');
      return;
    }

    try {
      setIsMinting(true);
      setError(null);
      setMintResult(null);

      // 验证钱包地址
      if (!walletAddress.trim()) {
        setError('请输入钱包地址');
        return;
      }

      const result = await faucetApi.mintTokens(amount, walletAddress);
      setMintResult(result);
      
      // Refresh user data to update quotas
      await refreshUser();
    } catch (err: any) {
      setError(err.response?.data?.message || '领取失败，请重试');
    } finally {
      setIsMinting(false);
    }
  };

  const handleCustomMint = () => {
    if (!isAuthenticated) {
      setError('请先登录');
      return;
    }

    const amount = parseFloat(customAmount);
    if (isNaN(amount) || amount <= 0) {
      setError('请输入有效的数量');
      return;
    }
    
    const amountInSmallestUnit = Math.floor(amount * 1e8);
    if (amountInSmallestUnit > (user?.max_amount || 0)) {
      setError(`数量不能超过最大限制 ${formatAmount(user?.max_amount || 0)}`);
      return;
    }

    handleMint(amountInSmallestUnit);
  };


  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircle className="h-5 w-5 text-green-500" />;
      case 'failed':
        return <XCircle className="h-5 w-5 text-red-500" />;
      case 'processing':
        return <RefreshCw className="h-5 w-5 text-blue-500 animate-spin" />;
      default:
        return <Clock className="h-5 w-5 text-yellow-500" />;
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'completed':
        return '已完成';
      case 'failed':
        return '失败';
      case 'processing':
        return '处理中';
      default:
        return '等待中';
    }
  };

  useEffect(() => {
    // 初始化 Google Auth 并渲染按钮
    const initGoogleAuth = async () => {
      try {
        await googleAuth.initialize();
        if (googleButtonRef.current && !isAuthenticated) {
          googleAuth.renderButton(googleButtonRef.current.id);
        }
      } catch (err) {
        // 忽略初始化错误
      }
    };

    if (!isAuthenticated) {
      initGoogleAuth();
    }
  }, [isAuthenticated]);

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
      {/* Header */}
      <header className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div className="flex items-center">
              <div className="h-10 w-10 bg-primary-100 rounded-full flex items-center justify-center">
                <Droplets className="h-6 w-6 text-primary-600" />
              </div>
              <h1 className="ml-3 text-2xl font-bold text-gray-900">水龙头服务</h1>
            </div>
            
            {isAuthenticated && user ? (
              <div className="flex items-center space-x-4">
                <div className="flex items-center space-x-2">
                  <User className="h-5 w-5 text-gray-500" />
                  <span className="text-sm text-gray-700">{user.handle}</span>
                  <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 capitalize">
                    {user.role}
                  </span>
                </div>
                <button
                  onClick={signOut}
                  className="text-sm text-gray-500 hover:text-gray-700"
                >
                  退出登录
                </button>
              </div>
            ) : (
              <div className="flex items-center space-x-4">
                <span className="text-sm text-gray-500">请登录以使用服务</span>
              </div>
            )}
          </div>
        </div>
      </header>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Hero Section */}
        <div className="text-center mb-12">
          <h2 className="text-4xl font-bold text-gray-900 mb-4">
            免费领取 Aptos 代币
          </h2>
          <p className="text-xl text-gray-600 mb-8">
            开始您的区块链之旅，免费获取测试代币进行开发和实验
          </p>
          
          {/* Features */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-12">
            <div className="text-center">
              <div className="mx-auto h-12 w-12 bg-green-100 rounded-full flex items-center justify-center mb-4">
                <Zap className="h-6 w-6 text-green-600" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">快速领取</h3>
              <p className="text-gray-600">一键领取测试代币，无需复杂设置</p>
            </div>
            <div className="text-center">
              <div className="mx-auto h-12 w-12 bg-blue-100 rounded-full flex items-center justify-center mb-4">
                <Shield className="h-6 w-6 text-blue-600" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">安全可靠</h3>
              <p className="text-gray-600">使用 Google 账户安全登录</p>
            </div>
            <div className="text-center">
              <div className="mx-auto h-12 w-12 bg-purple-100 rounded-full flex items-center justify-center mb-4">
                <Star className="h-6 w-6 text-purple-600" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">完全免费</h3>
              <p className="text-gray-600">无需任何费用，完全免费使用</p>
            </div>
          </div>
        </div>

        {/* Main Content */}
        <div className="max-w-4xl mx-auto">
          {!isAuthenticated ? (
            /* Login Section */
            <div className="text-center mb-12">
              <div className="card p-8 max-w-md mx-auto">
                <div className="mx-auto h-16 w-16 bg-primary-100 rounded-full flex items-center justify-center mb-6">
                  <LogIn className="h-8 w-8 text-primary-600" />
                </div>
                <h3 className="text-xl font-semibold text-gray-900 mb-4">登录账户</h3>
                <p className="text-gray-600 mb-6">
                  使用您的 Google 账户登录以开始领取代币
                </p>
                
                <div className="mb-4">
                  <div 
                    ref={googleButtonRef}
                    id="google-signin-button"
                    className="w-full"
                  ></div>
                </div>

                <p className="text-xs text-gray-500">
                  登录即表示您同意我们的服务条款和隐私政策
                </p>
              </div>
            </div>
          ) : (
            /* Authenticated User Interface - Step by Step Flow */
            <div className="space-y-6">
              {/* Step 1: Wallet Address Input - Most Important */}
              <div className="card p-6">
                <div className="flex items-center mb-4">
                  <div className="flex items-center justify-center w-8 h-8 bg-primary-100 text-primary-600 rounded-full text-sm font-semibold mr-3">
                    1
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900">输入钱包地址</h3>
                </div>
                
                <div className="relative">
                  <Wallet className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
                  <input
                    type="text"
                    value={walletAddress}
                    onChange={(e) => setWalletAddress(e.target.value)}
                    placeholder="输入钱包地址 (0x...)"
                    className="input pl-12 w-full text-lg"
                  />
                </div>
                <p className="text-sm text-gray-500 mt-2">
                  请输入有效的钱包地址，代币将发送到此地址
                </p>
              </div>

              {/* Step 2: Quick Actions - Only show if wallet address is provided */}
              {walletAddress.trim() && (
                <div className="card p-6">
                  <div className="flex items-center mb-4">
                    <div className="flex items-center justify-center w-8 h-8 bg-primary-100 text-primary-600 rounded-full text-sm font-semibold mr-3">
                      2
                    </div>
                    <h3 className="text-lg font-semibold text-gray-900">选择领取方式</h3>
                  </div>
                  
                  {error && (
                    <div className="mb-4 bg-red-50 border border-red-200 rounded-md p-4">
                      <div className="flex">
                        <AlertCircle className="h-5 w-5 text-red-400" />
                        <div className="ml-3">
                          <p className="text-sm text-red-800">{error}</p>
                        </div>
                      </div>
                    </div>
                  )}

                  {mintResult && (
                    <div className="mb-4 bg-green-50 border border-green-200 rounded-md p-4">
                      <div className="flex items-center">
                        {getStatusIcon(mintResult.status)}
                        <div className="ml-3">
                          <p className="text-sm font-medium text-green-800">
                            领取状态: {getStatusText(mintResult.status)}
                          </p>
                          <p className="text-sm text-green-700">
                            数量: {formatAmount(mintResult.amount)} APT
                          </p>
                          {mintResult.tx_hash && (
                            <p className="text-sm text-green-700">
                              交易哈希: {mintResult.tx_hash}
                            </p>
                          )}
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Quick mint buttons */}
                  <div className="mb-6">
                    <label className="block text-sm font-medium text-gray-700 mb-3">
                      快速领取
                    </label>
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                      <button
                        onClick={() => handleMint(100000000)}
                        disabled={isMinting || (user?.remaining_today !== undefined && user.remaining_today <= 0)}
                        className="btn btn-primary btn-lg disabled:opacity-50 flex items-center justify-center"
                      >
                        <Droplets className="h-5 w-5 mr-2" />
                        <div className="text-left">
                          <div className="font-semibold">默认数量</div>
                          <div className="text-sm opacity-90">1.00 APT</div>
                        </div>
                      </button>
                      <button
                        onClick={() => handleMint(user?.max_amount)}
                        disabled={isMinting || (user?.remaining_today !== undefined && user.remaining_today < (user?.max_amount || 0))}
                        className="btn btn-outline btn-lg disabled:opacity-50 flex items-center justify-center"
                      >
                        <Zap className="h-5 w-5 mr-2" />
                        <div className="text-left">
                          <div className="font-semibold">最大数量</div>
                          <div className="text-sm opacity-90">{formatAmount(user?.max_amount || 0)} APT</div>
                        </div>
                      </button>
                    </div>
                  </div>

                  {/* Custom amount */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-3">
                      自定义数量
                    </label>
                    <div className="flex space-x-3">
                      <input
                        type="number"
                        value={customAmount}
                        onChange={(e) => setCustomAmount(e.target.value)}
                        placeholder="输入数量 (APT)"
                        step="0.01"
                        min="0"
                        max={formatAmount(user?.max_amount || 0)}
                        className="input flex-1 text-lg"
                      />
                      <button
                        onClick={handleCustomMint}
                        disabled={isMinting || !customAmount}
                        className="btn btn-primary btn-lg disabled:opacity-50 flex items-center px-8"
                      >
                        <Coins className="h-5 w-5 mr-2" />
                        领取
                      </button>
                    </div>
                    <p className="text-xs text-gray-500 mt-2">
                      最大: {formatAmount(user?.max_amount || 0)} APT
                    </p>
                  </div>
                </div>
              )}

              {/* Step 3: User Info - Always visible but less prominent */}
              <div className="card p-6 bg-gray-50">
                <div className="flex items-center mb-4">
                  <div className="flex items-center justify-center w-8 h-8 bg-gray-200 text-gray-600 rounded-full text-sm font-semibold mr-3">
                    i
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900">账户信息</h3>
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                  <div className="text-center p-3 bg-white rounded-lg">
                    <div className="text-sm text-gray-600 mb-1">用户角色</div>
                    <div className="font-semibold capitalize text-primary-600">{user?.role}</div>
                  </div>
                  <div className="text-center p-3 bg-white rounded-lg">
                    <div className="text-sm text-gray-600 mb-1">最大单次领取</div>
                    <div className="font-semibold text-gray-900">{formatAmount(user?.max_amount || 0)} APT</div>
                  </div>
                  {user?.max_daily_cap && (
                    <div className="text-center p-3 bg-white rounded-lg">
                      <div className="text-sm text-gray-600 mb-1">每日限额</div>
                      <div className="font-semibold text-gray-900">{formatAmount(user.max_daily_cap)} APT</div>
                    </div>
                  )}
                  <div className="text-center p-3 bg-white rounded-lg">
                    <div className="text-sm text-gray-600 mb-1">今日已领取</div>
                    <div className="font-semibold text-gray-900">{formatAmount(user?.minted_today || 0)} APT</div>
                  </div>
                </div>
                
                {user?.remaining_today !== undefined && (
                  <div className="mt-4 p-4 bg-green-50 rounded-lg">
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium text-green-800">今日剩余</span>
                      <span className="text-lg font-bold text-green-600">
                        {formatAmount(user.remaining_today)} APT
                      </span>
                    </div>
                  </div>
                )}

                {/* Daily quota progress */}
                {user?.max_daily_cap && (
                  <div className="mt-4">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-medium text-gray-700">今日配额</span>
                      <span className="text-sm text-gray-600">
                        {formatAmount(user.minted_today)} / {formatAmount(user.max_daily_cap)} APT
                      </span>
                    </div>
                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div
                        className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                        style={{
                          width: `${Math.min((user.minted_today / user.max_daily_cap) * 100, 100)}%`
                        }}
                      ></div>
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default LandingPage;
