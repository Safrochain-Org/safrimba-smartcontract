# 🚀 **Frontend Integration Guide - Tontine Smart Contract**

## **📋 Overview**

This guide explains how to integrate the **Tontine Smart Contract** with a frontend application. The contract is built on **CosmWasm** and deployed on **Safrochain**, so we'll use **CosmJS** for blockchain interactions.

---

## **🏗️ Architecture Overview**

```
Frontend (React/Vue/Angular)
    ↓
CosmJS (Blockchain Client)
    ↓
Safrochain Network
    ↓
Tontine Smart Contract
```

---

## **📦 Required Dependencies**

### **1. Core Dependencies**

```bash
npm install @cosmjs/cosmwasm-stargate @cosmjs/proto-signing
npm install @cosmjs/amino @cosmjs/encoding
npm install @cosmjs/crypto @cosmjs/math
```

### **2. Additional Utilities**

```bash
npm install @cosmjs/launchpad @cosmjs/stargate
npm install @cosmjs/tendermint-rpc
npm install @cosmjs/keplr
```

---

## **⚙️ Environment Configuration**

### **1. Create `.env` File**

```bash
# Network Configuration
REACT_APP_CHAIN_ID=safro-testnet-1
REACT_APP_RPC_URL=https://rpc.testnet.safrochain.com
REACT_APP_REST_URL=https://rest.testnet.safrochain.com
REACT_APP_EXPLORER_URL=https://explorer.testnet.safrochain.com

# Contract Configuration
REACT_APP_CONTRACT_ADDRESS=addr_safro1q0lcq7g28hakev9d2ku7xy9634lml65xa0u5epte2uap9trjykqqqpquuy
REACT_APP_CODE_ID=29

# Gas Configuration
REACT_APP_GAS_PRICE=0.025usaf
REACT_APP_GAS_ADJUSTMENT=1.3

# Frontend Configuration
REACT_APP_APP_NAME=Tontine DApp
REACT_APP_APP_VERSION=1.0.0
REACT_APP_APP_DESCRIPTION=Decentralized Tontine Platform
```

### **2. Production Environment (`.env.production`)**

```bash
# Mainnet Configuration
REACT_APP_CHAIN_ID=safro-mainnet-1
REACT_APP_RPC_URL=https://rpc.mainnet.safrochain.com
REACT_APP_REST_URL=https://api.mainnet.safrochain.com
REACT_APP_EXPLORER_URL=https://explorer.testnet.safrochain.com

# Production Contract
REACT_APP_CONTRACT_ADDRESS=YOUR_MAINNET_CONTRACT_ADDRESS
REACT_APP_CODE_ID=YOUR_MAINNET_CODE_ID

# Production Gas Settings
REACT_APP_GAS_PRICE=0.025usaf
REACT_APP_GAS_ADJUSTMENT=1.2
```

---

## **🔧 Smart Contract Integration Setup**

### **1. Contract Client Configuration**

```typescript
// src/config/contract.ts
export const CONTRACT_CONFIG = {
  chainId: process.env.REACT_APP_CHAIN_ID || "safro-testnet-1",
  rpcUrl: process.env.REACT_APP_RPC_URL || "https://rpc.testnet.safrochain.com",
  restUrl:
    process.env.REACT_APP_REST_URL || "https://api.testnet.safrochain.com",
  contractAddress: process.env.REACT_APP_CONTRACT_ADDRESS || "",
  codeId: process.env.REACT_APP_CODE_ID || "29",
  gasPrice: process.env.REACT_APP_GAS_PRICE || "0.025usaf",
  gasAdjustment: parseFloat(process.env.REACT_APP_GAS_ADJUSTMENT || "1.3"),
};

export const NETWORK_CONFIG = {
  chainId: CONTRACT_CONFIG.chainId,
  chainName: "Safrochain",
  rpc: CONTRACT_CONFIG.rpcUrl,
  rest: CONTRACT_CONFIG.restUrl,
  bip44: {
    coinType: 118,
  },
  bech32Config: {
    bech32PrefixAccAddr: "safro",
    bech32PrefixAccPub: "safropub",
    bech32PrefixValAddr: "safrovaloper",
    bech32PrefixValPub: "safrovaloperpub",
    bech32PrefixConsAddr: "safrovalcons",
    bech32PrefixConsPub: "safrovalconspub",
  },
  currencies: [
    {
      coinDenom: "USAF",
      coinMinimalDenom: "usaf",
      coinDecimals: 6,
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "USAF",
      coinMinimalDenom: "usaf",
      coinDecimals: 6,
    },
  ],
  stakeCurrency: {
    coinDenom: "USAF",
    coinMinimalDenom: "usaf",
    coinDecimals: 6,
  },
  gasPriceStep: {
    low: 0.01,
    average: 0.025,
    high: 0.04,
  },
};
```

### **2. Contract Message Types**

```typescript
// src/types/contract.ts
export interface InstantiateMsg {
  admin: string;
  token_denom: string;
  contribution_amount: string;
  round_frequency: number;
  time_guards: number;
  beneficiaries: string[];
  late_penalty: string;
  protocol_fees: string;
  arbitrator: string;
}

export interface ExecuteMsg {
  register_member?: { address: string };
  start_tontine?: {};
  deposit_contribution?: {};
  distribute_to_beneficiary?: {};
  pause_tontine?: {};
  resume_tontine?: {};
  close_early?: { reason: string };
  advance_payment?: { beneficiary: string; discount: string };
  declare_late?: { member: string };
  apply_penalty?: { member: string; amount: string };
  pay_penalty?: { member: string };
  withdraw_fees?: {};
  collect_protocol_fees?: {};
  resolve_dispute?: { member: string; resolution: string };
  arbitrate_dispute?: { member: string; decision: string };
  finalize_tontine?: {};
}

export interface QueryMsg {
  get_config?: {};
  get_tontine_state?: {};
  get_members?: {};
  get_member?: { address: string };
  get_current_round?: {};
  get_round_info?: { round: number };
  get_tontine_balance?: {};
  get_accumulated_fees?: {};
  get_distribution_history?: {};
  get_deposit_history?: {};
  get_statistics?: {};
}
```



---

## **🔌 Blockchain Connection Setup**

### **1. Keplr Wallet Integration**

```typescript
// src/services/wallet.ts
import { Window as KeplrWindow } from "@keplr-wallet/types";
import {
  CosmWasmClient,
  SigningCosmWasmClient,
} from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { CONTRACT_CONFIG, NETWORK_CONFIG } from "../config/contract";

declare global {
  interface Window extends KeplrWindow {}
}

export class WalletService {
  private client: CosmWasmClient | null = null;
  private signingClient: SigningCosmWasmClient | null = null;
  private wallet: DirectSecp256k1HdWallet | null = null;

  async connectKeplr(): Promise<string> {
    if (!window.keplr) {
      throw new Error(
        "Keplr wallet not found. Please install Keplr extension."
      );
    }

    try {
      // Request connection to Safrochain
      await window.keplr.enable(CONTRACT_CONFIG.chainId);

      // Get account info
      const offlineSigner = window.keplr.getOfflineSigner(
        CONTRACT_CONFIG.chainId
      );
      const accounts = await offlineSigner.getAccounts();

      if (accounts.length === 0) {
        throw new Error("No accounts found in Keplr wallet.");
      }

      // Initialize clients
      this.client = await CosmWasmClient.connect(CONTRACT_CONFIG.rpcUrl);
      this.signingClient = await SigningCosmWasmClient.connectWithSigner(
        CONTRACT_CONFIG.rpcUrl,
        offlineSigner
      );

      return accounts[0].address;
    } catch (error) {
      console.error("Failed to connect Keplr:", error);
      throw error;
    }
  }

  async connectMnemonic(mnemonic: string): Promise<string> {
    try {
      this.wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
        prefix: "safro",
      });

      const accounts = await this.wallet.getAccounts();
      if (accounts.length === 0) {
        throw new Error("No accounts found in wallet.");
      }

      // Initialize clients
      this.client = await CosmWasmClient.connect(CONTRACT_CONFIG.rpcUrl);
      this.signingClient = await SigningCosmWasmClient.connectWithSigner(
        CONTRACT_CONFIG.rpcUrl,
        this.wallet
      );

      return accounts[0].address;
    } catch (error) {
      console.error("Failed to connect with mnemonic:", error);
      throw error;
    }
  }

  getClient(): CosmWasmClient | null {
    return this.client;
  }

  getSigningClient(): SigningCosmWasmClient | null {
    return this.signingClient;
  }

  async disconnect(): Promise<void> {
    this.client = null;
    this.signingClient = null;
    this.wallet = null;
  }
}
```

### **2. Contract Service**

```typescript
// src/services/contract.ts
import {
  CosmWasmClient,
  SigningCosmWasmClient,
} from "@cosmjs/cosmwasm-stargate";
import { ExecuteMsg, QueryMsg, InstantiateMsg } from "../types/contract";
import { CONTRACT_CONFIG } from "../config/contract";

export class ContractService {
  constructor(
    private client: CosmWasmClient,
    private signingClient: SigningCosmWasmClient | null = null
  ) {}

  // Query Methods
  async queryContract<T>(msg: QueryMsg): Promise<T> {
    try {
      const result = await this.client.queryContractSmart(
        CONTRACT_CONFIG.contractAddress,
        msg
      );
      return result as T;
    } catch (error) {
      console.error("Query failed:", error);
      throw error;
    }
  }

  // Execute Methods
  async executeContract(
    msg: ExecuteMsg,
    sender: string,
    funds?: { denom: string; amount: string }[]
  ): Promise<string> {
    if (!this.signingClient) {
      throw new Error("Signing client not available. Please connect wallet.");
    }

    try {
      const result = await this.signingClient.execute(
        sender,
        CONTRACT_CONFIG.contractAddress,
        msg,
        "auto",
        undefined,
        funds
      );

      return result.transactionHash;
    } catch (error) {
      console.error("Execute failed:", error);
      throw error;
    }
  }

  // Specific Contract Methods
  async getTontineState() {
    return this.queryContract({ get_tontine_state: {} });
  }

  async getMembers() {
    return this.queryContract({ get_members: {} });
  }

  async getCurrentRound() {
    return this.queryContract({ get_current_round: {} });
  }

  async getTontineBalance() {
    return this.queryContract({ get_tontine_balance: {} });
  }

  async registerMember(address: string, sender: string) {
    return this.executeContract({ register_member: { address } }, sender);
  }

  async startTontine(sender: string) {
    return this.executeContract({ start_tontine: {} }, sender);
  }

  async depositContribution(sender: string, amount: string) {
    const funds = [{ denom: "usaf", amount }];
    return this.executeContract({ deposit_contribution: {} }, sender, funds);
  }

  async distributeToBeneficiary(sender: string) {
    return this.executeContract({ distribute_to_beneficiary: {} }, sender);
  }
}
```

---

## **🎨 Frontend Components**

### **1. Wallet Connection Component**

```typescript
// src/components/WalletConnect.tsx
import React, { useState, useEffect } from "react";
import { WalletService } from "../services/wallet";

interface WalletConnectProps {
  onConnect: (address: string) => void;
  onDisconnect: () => void;
}

export const WalletConnect: React.FC<WalletConnectProps> = ({
  onConnect,
  onDisconnect,
}) => {
  const [isConnected, setIsConnected] = useState(false);
  const [address, setAddress] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  const walletService = new WalletService();

  const connectKeplr = async () => {
    setIsLoading(true);
    setError("");

    try {
      const connectedAddress = await walletService.connectKeplr();
      setAddress(connectedAddress);
      setIsConnected(true);
      onConnect(connectedAddress);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Connection failed");
    } finally {
      setIsLoading(false);
    }
  };

  const connectMnemonic = async (mnemonic: string) => {
    setIsLoading(true);
    setError("");

    try {
      const connectedAddress = await walletService.connectMnemonic(mnemonic);
      setAddress(connectedAddress);
      setIsConnected(true);
      onConnect(connectedAddress);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Connection failed");
    } finally {
      setIsLoading(false);
    }
  };

  const disconnect = async () => {
    await walletService.disconnect();
    setAddress("");
    setIsConnected(false);
    onDisconnect();
  };

  return (
    <div className="wallet-connect">
      {!isConnected ? (
        <div>
          <button
            onClick={connectKeplr}
            disabled={isLoading}
            className="btn btn-primary"
          >
            {isLoading ? "Connecting..." : "Connect Keplr"}
          </button>

          <div className="mt-3">
            <input
              type="text"
              placeholder="Or enter mnemonic phrase"
              className="form-control"
              onKeyPress={(e) => {
                if (e.key === "Enter") {
                  const target = e.target as HTMLInputElement;
                  connectMnemonic(target.value);
                }
              }}
            />
          </div>
        </div>
      ) : (
        <div>
          <p>Connected: {address}</p>
          <button onClick={disconnect} className="btn btn-secondary">
            Disconnect
          </button>
        </div>
      )}

      {error && <div className="alert alert-danger mt-2">{error}</div>}
    </div>
  );
};
```

### **2. Tontine Dashboard Component**

```typescript
// src/components/TontineDashboard.tsx
import React, { useState, useEffect } from "react";
import { ContractService } from "../services/contract";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

interface TontineDashboardProps {
  contractService: ContractService;
  userAddress: string;
}

export const TontineDashboard: React.FC<TontineDashboardProps> = ({
  contractService,
  userAddress,
}) => {
  const [tontineState, setTontineState] = useState(null);
  const [members, setMembers] = useState([]);
  const [currentRound, setCurrentRound] = useState(null);
  const [balance, setBalance] = useState("0");
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadTontineData();
  }, []);

  const loadTontineData = async () => {
    try {
      setIsLoading(true);

      const [state, membersList, round, tontineBalance] = await Promise.all([
        contractService.getTontineState(),
        contractService.getMembers(),
        contractService.getCurrentRound(),
        contractService.getTontineBalance(),
      ]);

      setTontineState(state);
      setMembers(membersList);
      setCurrentRound(round);
      setBalance(tontineBalance);
    } catch (error) {
      console.error("Failed to load tontine data:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleStartTontine = async () => {
    try {
      await contractService.startTontine(userAddress);
      await loadTontineData();
    } catch (error) {
      console.error("Failed to start tontine:", error);
    }
  };

  const handleDeposit = async (amount: string) => {
    try {
      await contractService.depositContribution(userAddress, amount);
      await loadTontineData();
    } catch (error) {
      console.error("Failed to deposit:", error);
    }
  };

  const handleDistribute = async () => {
    try {
      await contractService.distributeToBeneficiary(userAddress);
      await loadTontineData();
    } catch (error) {
      console.error("Failed to distribute:", error);
    }
  };

  if (isLoading) {
    return <div>Loading tontine data...</div>;
  }

  return (
    <div className="tontine-dashboard">
      <h2>Tontine Dashboard</h2>

      {/* Tontine State */}
      <div className="card mb-3">
        <div className="card-header">
          <h5>Tontine State</h5>
        </div>
        <div className="card-body">
          <p>Status: {tontineState?.is_active ? "Active" : "Inactive"}</p>
          <p>Current Round: {tontineState?.current_round || 0}</p>
          <p>Total Rounds: {tontineState?.total_rounds || 0}</p>
          <p>Total Balance: {balance} usaf</p>
        </div>
      </div>

      {/* Actions */}
      <div className="card mb-3">
        <div className="card-header">
          <h5>Actions</h5>
        </div>
        <div className="card-body">
          {!tontineState?.is_active && (
            <button
              onClick={handleStartTontine}
              className="btn btn-primary me-2"
            >
              Start Tontine
            </button>
          )}

          <button
            onClick={() => handleDeposit("5000000")}
            className="btn btn-success me-2"
          >
            Deposit 5,000,000 usaf
          </button>

          {currentRound?.state === "active" && (
            <button onClick={handleDistribute} className="btn btn-warning">
              Distribute to Beneficiary
            </button>
          )}
        </div>
      </div>

      {/* Members */}
      <div className="card mb-3">
        <div className="card-header">
          <h5>Members ({members.length})</h5>
        </div>
        <div className="card-body">
          {members.map((member, index) => (
            <div key={index} className="member-item">
              <p>Address: {member.address}</p>
              <p>Balance: {member.balance} usaf</p>
              <p>Status: {member.status}</p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
```

---

## **🚀 Complete Integration Example**

### **1. Main App Component**

```typescript
// src/App.tsx
import React, { useState } from "react";
import { WalletService } from "./services/wallet";
import { ContractService } from "./services/contract";
import { WalletConnect } from "./components/WalletConnect";
import { TontineDashboard } from "./components/TontineDashboard";
import { CONTRACT_CONFIG } from "./config/contract";
import "./App.css";

function App() {
  const [walletService] = useState(() => new WalletService());
  const [contractService, setContractService] =
    useState<ContractService | null>(null);
  const [userAddress, setUserAddress] = useState("");

  const handleWalletConnect = async (address: string) => {
    setUserAddress(address);

    const client = walletService.getClient();
    const signingClient = walletService.getSigningClient();

    if (client && signingClient) {
      const contract = new ContractService(client, signingClient);
      setContractService(contract);
    }
  };

  const handleWalletDisconnect = () => {
    setUserAddress("");
    setContractService(null);
  };

  return (
    <div className="App">
      <header className="App-header">
        <h1>Tontine DApp</h1>
        <WalletConnect
          onConnect={handleWalletConnect}
          onDisconnect={handleWalletDisconnect}
        />
      </header>

      <main>
        {contractService && userAddress ? (
          <TontineDashboard
            contractService={contractService}
            userAddress={userAddress}
          />
        ) : (
          <div className="welcome-message">
            <h2>Welcome to Tontine DApp</h2>
            <p>Connect your wallet to start managing tontines on Safrochain.</p>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
```

### **2. Environment Setup Script**

```bash
#!/bin/bash
# setup-env.sh

echo "Setting up Tontine DApp environment..."

# Create environment files
cat > .env << EOF
# Network Configuration
REACT_APP_CHAIN_ID=safro-testnet-1
REACT_APP_RPC_URL=https://rpc.testnet.safrochain.com
REACT_APP_REST_URL=https://api.testnet.safrochain.com
REACT_APP_EXPLORER_URL=https://explorer.testnet.safrochain.com

# Contract Configuration
REACT_APP_CONTRACT_ADDRESS=addr_safro1q0lcq7g28hakev9d2ku7xy9634lml65xa0u5epte2uap9trjykqqqpquuy
REACT_APP_CODE_ID=29

# Gas Configuration
REACT_APP_GAS_PRICE=0.025usaf
REACT_APP_GAS_ADJUSTMENT=1.3

# Frontend Configuration
REACT_APP_APP_NAME=Tontine DApp
REACT_APP_APP_VERSION=1.0.0
REACT_APP_APP_DESCRIPTION=Decentralized Tontine Platform
EOF

echo "Environment file created: .env"
echo "Please update contract addresses for your deployment."
```

---

## **📱 Mobile & Responsive Considerations**

### **1. Mobile Wallet Integration**

```typescript
// Mobile wallet detection
const isMobile = () => {
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
    navigator.userAgent
  );
};

// Mobile-specific wallet connection
const connectMobileWallet = async () => {
  if (isMobile()) {
    // Use mobile-specific wallet connection
    // Keplr Mobile, Cosmostation, etc.
  }
};
```

### **2. Responsive Design**

```css
/* Mobile-first responsive design */
.wallet-connect {
  padding: 1rem;
}

@media (min-width: 768px) {
  .wallet-connect {
    padding: 2rem;
  }
}

.tontine-dashboard {
  max-width: 100%;
  overflow-x: auto;
}
```

---

## **🔒 Security Considerations**

### **1. Input Validation**

```typescript
// Validate addresses
const isValidAddress = (address: string): boolean => {
  return address.startsWith("safro") && address.length > 10;
};

// Validate amounts
const isValidAmount = (amount: string): boolean => {
  const num = parseFloat(amount);
  return !isNaN(num) && num > 0;
};
```

### **2. Error Handling**

```typescript
// Global error handler
const handleContractError = (error: any): string => {
  if (error.message?.includes("insufficient funds")) {
    return "Insufficient funds for transaction";
  }
  if (error.message?.includes("unauthorized")) {
    return "Unauthorized action";
  }
  return "Transaction failed. Please try again.";
};
```

---

## **🧪 Testing & Development**

### **1. Development Commands**

```bash
# Install dependencies
npm install

# Start development server
npm start

# Build for production
npm run build

# Run tests
npm test

# Environment setup
chmod +x setup-env.sh
./setup-env.sh
```

### **2. Testing Environment**

```bash
# Test environment variables
REACT_APP_CHAIN_ID=safro-testnet-1
REACT_APP_RPC_URL=https://rpc.testnet.safrochain.com
REACT_APP_CONTRACT_ADDRESS=YOUR_TEST_CONTRACT
```

---

## **📚 Additional Resources**

### **1. Documentation**

- [CosmJS Documentation](https://cosmos.github.io/cosmjs/)
- [CosmWasm Documentation](https://docs.cosmwasm.com/)
- [Keplr Wallet Documentation](https://docs.keplr.app/)

### **2. Examples**

- [CosmJS Examples](https://github.com/cosmos/cosmjs/tree/main/packages/samples)
- [CosmWasm Examples](https://github.com/CosmWasm/cw-examples)

### **3. Community**

- [Cosmos Discord](https://discord.gg/cosmosnetwork)
- [CosmWasm Discord](https://discord.gg/cosmwasm)

---

## **🎯 Next Steps**

1. **Set up environment variables** using the provided script
2. **Install dependencies** for your chosen frontend framework
3. **Integrate wallet connection** using Keplr or mnemonic
4. **Connect to contract** using the ContractService
5. **Build UI components** for tontine management
6. **Test thoroughly** on testnet before mainnet deployment
7. **Deploy frontend** to your hosting platform

---

_This guide provides a complete foundation for integrating the Tontine Smart Contract with any frontend framework. Customize the components and styling according to your application's needs._
