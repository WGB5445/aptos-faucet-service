import React, { useState } from 'react';
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
  Calendar,
  Zap
} from 'lucide-react';

const HomePage: React.FC = () => {
  const { user, refreshUser } = useAuth();
  const [isMinting, setIsMinting] = useState(false);
  const [mintResult, setMintResult] = useState<MintResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [customAmount, setCustomAmount] = useState<string>('');

  const formatAmount = (amount: number) => {
    return (amount / 1e8).toFixed(2);
  };


  const handleMint = async (amount?: number) => {
    try {
      setIsMinting(true);
      setError(null);
      setMintResult(null);

      const result = await faucetApi.mintTokens(amount);
      setMintResult(result);
      
      // Refresh user data to update quotas
      await refreshUser();
    } catch (err: any) {
      console.error('Mint failed:', err);
      setError(err.response?.data?.message || '领取失败，请重试');
    } finally {
      setIsMinting(false);
    }
  };

  const handleCustomMint = () => {
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

  if (!user) return null;

  return (
    <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      {/* Header */}
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">
          欢迎使用水龙头服务
        </h1>
        <p className="text-gray-600">
          领取免费的代币，开始您的区块链之旅
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* User Stats */}
        <div className="lg:col-span-1">
          <div className="card p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">账户信息</h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600">用户角色</span>
                <span className="text-sm font-medium capitalize">{user.role}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600">最大单次领取</span>
                <span className="text-sm font-medium">{formatAmount(user.max_amount)} APT</span>
              </div>
              {user.max_daily_cap && (
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-600">每日限额</span>
                  <span className="text-sm font-medium">{formatAmount(user.max_daily_cap)} APT</span>
                </div>
              )}
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600">今日已领取</span>
                <span className="text-sm font-medium">{formatAmount(user.minted_today)} APT</span>
              </div>
              {user.remaining_today !== undefined && (
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-600">今日剩余</span>
                  <span className="text-sm font-medium text-green-600">
                    {formatAmount(user.remaining_today)} APT
                  </span>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Mint Interface */}
        <div className="lg:col-span-2">
          <div className="card p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">领取代币</h3>
            
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

            <div className="space-y-4">
              {/* Quick mint buttons */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  快速领取
                </label>
                <div className="grid grid-cols-2 gap-3">
                  <button
                    onClick={() => handleMint()}
                    disabled={isMinting || (user.remaining_today !== undefined && user.remaining_today <= 0)}
                    className="btn btn-primary btn-md disabled:opacity-50"
                  >
                    <Droplets className="h-4 w-4 mr-2" />
                    默认数量
                  </button>
                  <button
                    onClick={() => handleMint(user.max_amount)}
                    disabled={isMinting || (user.remaining_today !== undefined && user.remaining_today < user.max_amount)}
                    className="btn btn-outline btn-md disabled:opacity-50"
                  >
                    <Zap className="h-4 w-4 mr-2" />
                    最大数量
                  </button>
                </div>
              </div>

              {/* Custom amount */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  自定义数量
                </label>
                <div className="flex space-x-2">
                  <input
                    type="number"
                    value={customAmount}
                    onChange={(e) => setCustomAmount(e.target.value)}
                    placeholder="输入数量 (APT)"
                    step="0.01"
                    min="0"
                    max={formatAmount(user.max_amount)}
                    className="input flex-1"
                  />
                  <button
                    onClick={handleCustomMint}
                    disabled={isMinting || !customAmount}
                    className="btn btn-primary btn-md disabled:opacity-50"
                  >
                    <Coins className="h-4 w-4 mr-2" />
                    领取
                  </button>
                </div>
                <p className="text-xs text-gray-500 mt-1">
                  最大: {formatAmount(user.max_amount)} APT
                </p>
              </div>
            </div>
          </div>

          {/* Daily quota info */}
          {user.max_daily_cap && (
            <div className="mt-6 card p-4">
              <div className="flex items-center">
                <Calendar className="h-5 w-5 text-blue-500 mr-2" />
                <div>
                  <p className="text-sm font-medium text-gray-900">今日配额</p>
                  <p className="text-sm text-gray-600">
                    已使用 {formatAmount(user.minted_today)} / {formatAmount(user.max_daily_cap)} APT
                  </p>
                  <div className="mt-2 w-full bg-gray-200 rounded-full h-2">
                    <div
                      className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                      style={{
                        width: `${Math.min((user.minted_today / user.max_daily_cap) * 100, 100)}%`
                      }}
                    ></div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default HomePage;
