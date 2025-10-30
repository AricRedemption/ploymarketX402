// React组件示例 - 预测市场界面
import React, { useState, useEffect } from 'react';
import { PublicKey } from '@solana/web3.js';
import { 
  usePredictionMarket, 
  useMarketInfo, 
  useUserInfo, 
  useSwapPreview,
  usePredictionMarketUtils 
} from './hooks/usePredictionMarket';
import { TokenType, SwapDirection } from './PredictionMarketClient';

// 市场卡片组件
interface MarketCardProps {
  marketPDA: PublicKey;
  onSelect?: (marketPDA: PublicKey) => void;
}

export function MarketCard({ marketPDA, onSelect }: MarketCardProps) {
  const { marketInfo, loading, error } = useMarketInfo(marketPDA);
  const { calculatePrice, formatTokenAmount } = usePredictionMarketUtils();

  if (loading) {
    return (
      <div className="market-card loading">
        <div className="skeleton">加载中...</div>
      </div>
    );
  }

  if (error || !marketInfo) {
    return (
      <div className="market-card error">
        <div className="error-message">加载失败: {error}</div>
      </div>
    );
  }

  const { yesPrice, noPrice } = calculatePrice(
    marketInfo.realYesSolReserves.toNumber(),
    marketInfo.realNoSolReserves.toNumber()
  );

  return (
    <div className="market-card" onClick={() => onSelect?.(marketPDA)}>
      <div className="market-header">
        <h3>预测市场</h3>
        <span className={`status ${marketInfo.isCompleted ? 'completed' : 'active'}`}>
          {marketInfo.isCompleted ? '已结束' : '进行中'}
        </span>
      </div>
      
      <div className="market-content">
        <div className="token-info">
          <div className="token-row">
            <span className="token-name">YES</span>
            <span className="token-price">{(yesPrice * 100).toFixed(1)}%</span>
            <span className="token-reserves">
              {formatTokenAmount(marketInfo.realYesSolReserves)} SOL
            </span>
          </div>
          <div className="token-row">
            <span className="token-name">NO</span>
            <span className="token-price">{(noPrice * 100).toFixed(1)}%</span>
            <span className="token-reserves">
              {formatTokenAmount(marketInfo.realNoSolReserves)} SOL
            </span>
          </div>
        </div>
        
        <div className="market-stats">
          <div className="stat">
            <span className="label">总流动性:</span>
            <span className="value">
              {formatTokenAmount(marketInfo.totalLpAmount)} SOL
            </span>
          </div>
          <div className="stat">
            <span className="label">LP数量:</span>
            <span className="value">{marketInfo.lps.length}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

// 交易界面组件
interface TradingInterfaceProps {
  marketPDA: PublicKey;
}

export function TradingInterface({ marketPDA }: TradingInterfaceProps) {
  const { swapTokens, addLiquidity, withdrawLiquidity, userInfo } = usePredictionMarket({});
  const { marketInfo } = useMarketInfo(marketPDA);
  const { userInfo: currentUserInfo } = useUserInfo(marketPDA);
  const { formatTokenAmount, parseTokenAmount } = usePredictionMarketUtils();

  const [amount, setAmount] = useState('');
  const [tokenType, setTokenType] = useState<TokenType>(TokenType.YES);
  const [direction, setDirection] = useState<SwapDirection>(SwapDirection.BUY);
  const [liquidityAmount, setLiquidityAmount] = useState('');
  const [loading, setLoading] = useState(false);

  const { preview } = useSwapPreview(marketPDA, parseFloat(amount) || 0, tokenType);

  const handleSwap = async () => {
    if (!amount || !marketInfo) return;
    
    try {
      setLoading(true);
      const parsedAmount = parseTokenAmount(amount);
      
      await swapTokens(marketPDA, {
        amount: parsedAmount,
        direction,
        tokenType,
        minimumReceiveAmount: 0 // 可以根据预览计算
      });
      
      setAmount('');
    } catch (error) {
      console.error('交易失败:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleAddLiquidity = async () => {
    if (!liquidityAmount || !marketInfo) return;
    
    try {
      setLoading(true);
      const parsedAmount = parseTokenAmount(liquidityAmount);
      
      await addLiquidity(marketPDA, parsedAmount);
      
      setLiquidityAmount('');
    } catch (error) {
      console.error('添加流动性失败:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleWithdrawLiquidity = async () => {
    if (!liquidityAmount || !marketInfo) return;
    
    try {
      setLoading(true);
      const parsedAmount = parseTokenAmount(liquidityAmount);
      
      await withdrawLiquidity(marketPDA, parsedAmount);
      
      setLiquidityAmount('');
    } catch (error) {
      console.error('提取流动性失败:', error);
    } finally {
      setLoading(false);
    }
  };

  if (!marketInfo) {
    return <div>加载市场信息中...</div>;
  }

  return (
    <div className="trading-interface">
      <div className="trading-header">
        <h2>交易界面</h2>
        <div className="market-summary">
          <div className="summary-item">
            <span>YES价格: {(preview?.buyResult?.newYesReserves / (preview?.buyResult?.newYesReserves + preview?.buyResult?.newNoReserves) * 100).toFixed(1)}%</span>
          </div>
          <div className="summary-item">
            <span>NO价格: {(preview?.buyResult?.newNoReserves / (preview?.buyResult?.newYesReserves + preview?.buyResult?.newNoReserves) * 100).toFixed(1)}%</span>
          </div>
        </div>
      </div>

      <div className="trading-sections">
        {/* 交易部分 */}
        <div className="trading-section">
          <h3>交易代币</h3>
          
          <div className="form-group">
            <label>代币类型:</label>
            <select 
              value={tokenType} 
              onChange={(e) => setTokenType(Number(e.target.value) as TokenType)}
            >
              <option value={TokenType.YES}>YES</option>
              <option value={TokenType.NO}>NO</option>
            </select>
          </div>

          <div className="form-group">
            <label>交易方向:</label>
            <select 
              value={direction} 
              onChange={(e) => setDirection(Number(e.target.value) as SwapDirection)}
            >
              <option value={SwapDirection.BUY}>买入</option>
              <option value={SwapDirection.SELL}>卖出</option>
            </select>
          </div>

          <div className="form-group">
            <label>数量 (SOL):</label>
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="输入交易数量"
              step="0.001"
            />
          </div>

          {preview && (
            <div className="preview">
              <h4>交易预览:</h4>
              <div className="preview-item">
                <span>代币数量: {formatTokenAmount(preview.buyResult?.tokenAmount || 0)}</span>
              </div>
              <div className="preview-item">
                <span>SOL变化: {formatTokenAmount(preview.buyResult?.changeAmount || 0)}</span>
              </div>
            </div>
          )}

          <button 
            onClick={handleSwap} 
            disabled={loading || !amount}
            className="btn btn-primary"
          >
            {loading ? '交易中...' : '执行交易'}
          </button>
        </div>

        {/* 流动性部分 */}
        <div className="trading-section">
          <h3>流动性管理</h3>
          
          {currentUserInfo?.isLp ? (
            <div className="lp-info">
              <p>您是此市场的流动性提供者</p>
              <div className="form-group">
                <label>提取数量 (SOL):</label>
                <input
                  type="number"
                  value={liquidityAmount}
                  onChange={(e) => setLiquidityAmount(e.target.value)}
                  placeholder="输入提取数量"
                  step="0.001"
                />
              </div>
              <button 
                onClick={handleWithdrawLiquidity} 
                disabled={loading || !liquidityAmount}
                className="btn btn-secondary"
              >
                {loading ? '提取中...' : '提取流动性'}
              </button>
            </div>
          ) : (
            <div className="add-liquidity">
              <div className="form-group">
                <label>添加数量 (SOL):</label>
                <input
                  type="number"
                  value={liquidityAmount}
                  onChange={(e) => setLiquidityAmount(e.target.value)}
                  placeholder="输入添加数量"
                  step="0.001"
                />
              </div>
              <button 
                onClick={handleAddLiquidity} 
                disabled={loading || !liquidityAmount}
                className="btn btn-primary"
              >
                {loading ? '添加中...' : '添加流动性'}
              </button>
            </div>
          )}
        </div>

        {/* 用户持仓 */}
        <div className="trading-section">
          <h3>我的持仓</h3>
          {currentUserInfo ? (
            <div className="user-positions">
              <div className="position-item">
                <span className="label">YES余额:</span>
                <span className="value">{formatTokenAmount(currentUserInfo.yesBalance)}</span>
              </div>
              <div className="position-item">
                <span className="label">NO余额:</span>
                <span className="value">{formatTokenAmount(currentUserInfo.noBalance)}</span>
              </div>
              <div className="position-item">
                <span className="label">LP状态:</span>
                <span className="value">{currentUserInfo.isLp ? '是' : '否'}</span>
              </div>
            </div>
          ) : (
            <p>暂无持仓信息</p>
          )}
        </div>
      </div>
    </div>
  );
}

// 创建市场组件
export function CreateMarketForm() {
  const { createMarket, loading, error } = usePredictionMarket({});
  const [formData, setFormData] = useState({
    yesSymbol: '',
    yesUri: '',
    startSlot: '',
    endingSlot: ''
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    try {
      await createMarket({
        yesSymbol: formData.yesSymbol,
        yesUri: formData.yesUri,
        startSlot: formData.startSlot ? parseInt(formData.startSlot) : undefined,
        endingSlot: formData.endingSlot ? parseInt(formData.endingSlot) : undefined
      });
      
      // 重置表单
      setFormData({
        yesSymbol: '',
        yesUri: '',
        startSlot: '',
        endingSlot: ''
      });
    } catch (error) {
      console.error('创建市场失败:', error);
    }
  };

  return (
    <div className="create-market-form">
      <h2>创建预测市场</h2>
      
      {error && (
        <div className="error-message">
          错误: {error}
        </div>
      )}
      
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label>YES代币符号:</label>
          <input
            type="text"
            value={formData.yesSymbol}
            onChange={(e) => setFormData({ ...formData, yesSymbol: e.target.value })}
            placeholder="例如: YES"
            required
          />
        </div>

        <div className="form-group">
          <label>YES代币元数据URI:</label>
          <input
            type="url"
            value={formData.yesUri}
            onChange={(e) => setFormData({ ...formData, yesUri: e.target.value })}
            placeholder="https://example.com/metadata.json"
            required
          />
        </div>

        <div className="form-group">
          <label>开始槽位 (可选):</label>
          <input
            type="number"
            value={formData.startSlot}
            onChange={(e) => setFormData({ ...formData, startSlot: e.target.value })}
            placeholder="留空表示立即开始"
          />
        </div>

        <div className="form-group">
          <label>结束槽位 (可选):</label>
          <input
            type="number"
            value={formData.endingSlot}
            onChange={(e) => setFormData({ ...formData, endingSlot: e.target.value })}
            placeholder="留空表示无结束时间"
          />
        </div>

        <button 
          type="submit" 
          disabled={loading}
          className="btn btn-primary"
        >
          {loading ? '创建中...' : '创建市场'}
        </button>
      </form>
    </div>
  );
}

// 主应用组件
export function PredictionMarketApp() {
  const [selectedMarket, setSelectedMarket] = useState<PublicKey | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  
  // 这里应该从某个地方获取市场列表
  const markets: PublicKey[] = []; // 实际应用中需要从合约或索引器获取

  return (
    <div className="prediction-market-app">
      <header className="app-header">
        <h1>Solana 预测市场</h1>
        <div className="header-actions">
          <button 
            onClick={() => setShowCreateForm(!showCreateForm)}
            className="btn btn-primary"
          >
            {showCreateForm ? '取消创建' : '创建市场'}
          </button>
        </div>
      </header>

      <main className="app-main">
        {showCreateForm ? (
          <CreateMarketForm />
        ) : selectedMarket ? (
          <TradingInterface marketPDA={selectedMarket} />
        ) : (
          <div className="markets-grid">
            <h2>市场列表</h2>
            {markets.length > 0 ? (
              <div className="markets-list">
                {markets.map((marketPDA) => (
                  <MarketCard
                    key={marketPDA.toString()}
                    marketPDA={marketPDA}
                    onSelect={setSelectedMarket}
                  />
                ))}
              </div>
            ) : (
              <div className="empty-state">
                <p>暂无市场</p>
                <button 
                  onClick={() => setShowCreateForm(true)}
                  className="btn btn-primary"
                >
                  创建第一个市场
                </button>
              </div>
            )}
          </div>
        )}
      </main>
    </div>
  );
}

// CSS样式 (可以放在单独的CSS文件中)
const styles = `
.prediction-market-app {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
  padding-bottom: 20px;
  border-bottom: 1px solid #eee;
}

.markets-grid {
  display: grid;
  gap: 20px;
}

.markets-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.market-card {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s;
}

.market-card:hover {
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  transform: translateY(-2px);
}

.market-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.status {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: bold;
}

.status.active {
  background-color: #e8f5e8;
  color: #2d5a2d;
}

.status.completed {
  background-color: #f5e8e8;
  color: #5a2d2d;
}

.token-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.token-name {
  font-weight: bold;
}

.trading-interface {
  display: grid;
  gap: 30px;
}

.trading-sections {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
}

.trading-section {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 20px;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: bold;
  transition: all 0.2s;
}

.btn-primary {
  background-color: #007bff;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #0056b3;
}

.btn-secondary {
  background-color: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #545b62;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.preview {
  background-color: #f8f9fa;
  padding: 15px;
  border-radius: 4px;
  margin-bottom: 15px;
}

.preview-item {
  display: flex;
  justify-content: space-between;
  margin-bottom: 5px;
}

.user-positions {
  display: grid;
  gap: 10px;
}

.position-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid #eee;
}

.error-message {
  background-color: #f8d7da;
  color: #721c24;
  padding: 10px;
  border-radius: 4px;
  margin-bottom: 15px;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #6c757d;
}

.skeleton {
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: loading 1.5s infinite;
  height: 20px;
  border-radius: 4px;
}

@keyframes loading {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}
`;

// 导出样式
export { styles };
