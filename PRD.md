## Solana BNPL 智能合約系統需求規格書

### 1. 專案概述

**專案名稱**：Solana BNPL 智能合約系統

**目標**：在 Solana 區塊鏈上實現一個安全、高效的 BNPL 服務，允許用戶使用升息資產（如 jupSOL）獲得 xxUSD 代幣，並通過分期付款方式購買商品。

**核心功能**：

1. 資產存入與估值
2. xxUSD 鑄造與分配
3. 分期付款管理
4. SOL 贖回機制
5. 對沖策略資產管理

**開發環境**：

- 區塊鏈平台：Solana
- 開發語言：Rust
- 智能合約標準：Solana 程序

### **1.1 使用場景流程**

用戶使用BNPL服務，將 jupSOL 傳送到智能合約中，智能合約收到之後會鑄造與收到的 jupSOL 等值的 xxUSD。這些鑄造的 xxUSD會分成2部分，一部分與商品價值相等的 xxUSD 會被鎖定，用戶每天可以領回變動數量的 xxUSD，剩餘的 xxUSD 會在當下還給用戶，用在在鎖定期間到期後總共可以領回的 xxUSD 與他購買的商品價值一樣，之後用戶可以用他購買的商品價值一樣的 xxUSD來贖回當初他投入等量的 SOL，用戶的鎖定期間到期後，他如果選擇贖回 SOL，會需要從對沖策略那邊的資產取出對應數量轉換成 SOL 給用戶，用戶如果在時間到期後1個月都沒有執行贖回，那他就不能在贖回了，用戶用於贖回的 xxUSD 會燒毀。

而收到的 jupSOL 會集中到一個智能合約內投入對沖策略產生更多利息。

### 1.2 產品流程

1. 用戶將 jupSOL 傳送到智能合約中
    1. jupSOL={指定資產}
2. 智能合約估值並鑄造 xxUSD：
    - 計算用戶傳入的{指定資產}價值
        - 用戶傳入的{指定資產}價值 ={指定資產價格} * {用戶傳入的數量}
        - 需要使用 Oracle 取得{指定資產價格}
    - {新鑄造的xxUSD數量} = 本次鑄造的xxUSD數量
3. xxUSD 分配和鎖定：
    - 取得需要新鑄造的數量={新鑄造的xxUSD數量}
    - 計算{需要鎖定的xxUSD數量}={商品價格}/{xxUSD價格}
        - xxUSD 價格預設為 $1
        - 需要透過 API 取得商品價格
            - 目前暫時沒有 API，所以從前端取得 `{orderDetails}` 做為商品價格
    - 計算要{還給用戶的xxUSD數量}={新鑄造的xxUSD數量}-{需要鎖定的xxUSD數量}
    - 計算{需要鎖定的時間}
        - {需要鎖定的時間}={商品價格}*/(* {用戶傳入的數量}*{指定資產當前 APY}/365)
    - 計算**每天可以領回變動xxUSD數量 =** {需要鎖定的xxUSD數量} / {需要鎖定的時間}
    - 執行 {需要鎖定的xxUSD數量} xxUSD 鎖定
    - 將 {還給用戶的xxUSD數量} xxUSD 傳回給用戶
4. 用戶每天領回變動數量的 xxUSD：
    - 用戶可於網站上與合約互動執行領取
5. 鎖定期結束後，用戶可以贖回 SOL：
    - 確定鎖定期是否結束，如果否則無法執行贖回
    - 如果鎖定期已結束且沒有超過可提領時間，用戶可贖回 SOL
        - 可提領時間=鎖定期結束時間+14天
    - 取得當初用戶傳入的 {用戶傳入的數量}數量
    - 取得當初用戶鑄造的 {新鑄造的xxUSD數量} 數量
    - 用戶執行贖回，需要先傳入{新鑄造的xxUSD數量} 等量 xxUSD
    - 收到之後，將剛剛收到的{新鑄造的xxUSD數量} 等量 xxUSD燒毀
    - 計算{需要的取出的指定資產數量} =  最新 SOL 價格 * 當初{用戶傳入的數量} /最新{指定資產價格}
    - 取出{需要的取出的指定資產數量} SOL 並還給用戶
6. 對沖策略管理：
    - 接收用戶 {用戶傳入的數量} 的 {指定資產}
    - 接收後端對沖策略管理指令，下為指令
        - 將{指定資產}存入借貸平台
        - 將美金穩定幣存入借貸平台
            - USDC or PYUSD
        - 將{指定資產}從借貸平台取出
        - 將美金穩定幣從借貸平台取出
            - USDC or PYUSD
        - 將{指定資產}賣成美金穩定幣
        - 將{指定資產}賣成SOL
        - 使用美金穩定幣買入{指定資產}
        - 取出{指定資產}到指定地址
        - 取出SOL到指定地址
        - 取出美金穩定幣到指定地址
7. 價格查詢：
    - 取得價格
        - API: https://price.jup.ag/v6/price?ids={指定資產}[&vsToken=USDC](https://jup.ag/swap/JupSOL-USDC)
        - 得到的回應如下
            - {"data":{"JupSOL":{"id":"jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v","mintSymbol":"JupSOL","vsToken":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","vsTokenSymbol":"USDC","price":154.739119979}},"timeTaken":0.00030760999652557075}
            - 其中 "price"得到的數字 154.739 是我們要的指定資產價格
        - 指定資產格式
            - JitoSOL
            - mSOL
            - JupSOL
            - bSOL
            - vSOL
            - hSOL
        - 取回 SOL價格
            - API: https://price.jup.ag/v6/price?ids=SOL&vsToken=USDC
        - 取回 USDC價格
            - 先假設為 $1
        - 取回 USDC價格
            - 先假設為 $1
    - 獲取當前價格
- 取回當前 APY
    - {指定資產當前 APY} 透過 APY 取取得
        - API: https://sanctum-extra-api.ngrok.dev/v1/apy/latest?lst=bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1&lst=jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v&lst=he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A&lst=mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So&lst=J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn&lst=vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7
        - 回應如下
            - { "apys": { "vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7": 0.06764671820868433, "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v": 0.07930357606968097, "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1": 0.06704165547600575, "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So": 0.07609971048528023, "he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A": 0.07431500002983886, "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn": 0.07123133122896008 }, "errs": {} }
            - 其中vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7 為 vSOL
            - 其中 jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v 為 jupSOL
            - 其中 bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1 為 bSOL
            - 其中 mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So 為 mSOL
            - 其中 he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A 為 HSOL
            - 其中 J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn 為 jitoSOL
            - 而後的數字為當前的APY
- 以上取得價格與APY 的API已經被部屬在 switchboard oracle 上，以下是 devnet 上的數據源
    - bSOL數據源: 2CNMT1r5mWyWTYYD23UoV9ueq2cHpj2qvZKUwrP5LftU
    - mSOL數據源: mU2inw8URG5s97X8xFhY9y2VsLZSPrwqY3eky4DjEQQ
    - HSOL數據源: 4U1ofakLouLjVHXxMXXwNDkz3eUUSMXMqTyeC16Trpdf
    - JitoSOL數據源: 3UF281FMHbuXKsfqGQKQVExexnvMsHMGoGoo917rVf3g
    - JupSOL數據源: 3zkXukqF4CBSUAq55uAx1CnGrzDKk3cVAesJ4WLpSzgA
    - vSOL數據源: 2e2WhmSbWvNR94tXzK1caBEX1uCedXns6xVXfqePctJq
    - 價格與 APY數據源回覆格式如下
        
        ```jsx
        {
          "result": "77.4181388951796221050",
          "results": [
            "154.768612",
            "0.06766579035924421"
          ],
          "version": "RC_09_16_24_18_54"
        }
        ```
        
        - “results” 中的第一個數字是價格，第2個數字是APY
    - 取得 SOL 價格
        - SOL 數據源:  98tVEYkSyG7Di424o98ETFMTxocb5bCWARAeuUF1haL4
    - SOL 價格數據源回覆如下
        
        ```jsx
        {
          "result": "150.096956",
          "results": [
            "150.096956"
          ],
          "version": "RC_09_16_24_18_54"
        }
        ```
        
        - “result” 後的數字就是我們要的價格
- 取回商品價格
    - 目前還沒有 API 先預留
1. 權限管理和緊急停止：
    - 緊急停止
        - 停止 xxUSD 鑄造
            - 當更新可鑄造 xxUSD 總量低於 10000 時，觸發
            - 當{指定資產}價格與上一次更新價差達20%時，觸發
            - 當 xxUSD 價格低於 0.94 時，觸發
            - 手動觸發
        - 停止對沖策略管理資產轉移
            - 當轉移資產大於沖策略管理資產資產價值的25%時，觸發
            - 手動觸發
        - 停止對沖策略管理資產交易
            - 當交易資產大於沖策略管理資產資產價值的25%時，觸發
            - 手動觸發
        - 停止 xxUSD 燒毀
            - 手動觸發
    - 預防機制
        - 新增{指定資產}後，至少7天後才會生效
        - 更新可鑄造 xxUSD 總量
    - 權限管理
        - 執行商品價格傳入
        - 執行後端對沖策略管理指令
        - 新增/刪除{指定資產}
        - 更新合約
    - 其他安全相關的緊急停止、預防機制與權限管理