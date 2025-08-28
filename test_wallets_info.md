# 🧪 Test Wallets Information

## 📋 Wallet Details

### **Wallet 1: Admin et Premier Membre**

- **Nom** : `test_admin`
- **Adresse** : `addr_safro1g30jss4zxz5ra7efux45etpcmw7vk36vhqm47w`
- **Rôle** : Admin du contrat, premier bénéficiaire
- **Fonds requis** : 10,000,000 usaf (10 USAF)

### **Wallet 2: Deuxième Membre**

- **Nom** : `test_member2`
- **Adresse** : `addr_safro1djg2upp4enfm57wxc3h4ary8mddn36js33484h`
- **Rôle** : Membre de la tontine, deuxième bénéficiaire
- **Fonds requis** : 10,000,000 usaf (10 USAF)

### **Wallet 3: Troisième Membre**

- **Nom** : `test_member3`
- **Adresse** : `addr_safro12yx0z58fu69xgdq4kr4s4qsqgjpl8hf8lunm4w`
- **Rôle** : Membre de la tontine, troisième bénéficiaire
- **Fonds requis** : 10,000,000 usaf (10 USAF)

## 🔑 Keyring Commands

### **Lister tous les wallets**

```bash
safrochaind keys list --keyring-backend test
```

### **Vérifier le solde d'un wallet**

```bash
safrochaind query bank balances <ADDRESS> --node https://rpc.testnet.safrochain.com --output json
```

### **Exemple pour test_admin**

```bash
safrochaind query bank balances addr_safro1g30jss4zxz5ra7efux45etpcmw7vk36vhqm47w --node https://rpc.testnet.safrochain.com --output json
```

## 💰 Funding Instructions

Chaque wallet de test doit avoir **10,000,000 usaf** (10 USAF) pour les tests complets.

### **Méthodes de financement :**

1. **Faucet du testnet** (si disponible)
2. **Transfert depuis un wallet existant** avec des fonds
3. **Mining/validation** si vous avez un nœud

### **Vérification des soldes**

```bash
# Vérifier test_admin
safrochaind query bank balances addr_safro1g30jss4zxz5ra7efux45etpcmw7vk36vhqm47w --node https://rpc.testnet.safrochain.com --output json | jq '.balances[] | select(.denom=="usaf") | {address: "test_admin", balance: .amount + .denom}'

# Vérifier test_member2
safrochaind query bank balances addr_safro1djg2upp4enfm57wxc3h4ary8mddn36js33484h --node https://rpc.testnet.safrochain.com --output json | jq '.balances[] | select(.denom=="usaf") | {address: "test_member2", balance: .amount + .denom}'

# Vérifier test_member3
safrochaind query bank balances addr_safro12yx0z58fu69xgdq4kr4s4qsqgjpl8hf8lunm4w --node https://rpc.testnet.safrochain.com --output json | jq '.balances[] | select(.denom=="usaf") | {address: "test_member3", balance: .amount + .denom}'
```

## 🚀 Test Workflow

### **1. Script automatique complet**

```bash
./scripts/test_with_new_wallets.sh
```

### **2. Tests manuels après déploiement**

- Dépôts de contributions : `./scripts/05_deposit_contribution.sh`
- Distribution aux bénéficiaires : `./scripts/07_distribute_beneficiary.sh`
- Queries d'état : `./scripts/06_query_state.sh`

## 📝 Configuration de la Tontine

### **Paramètres d'instanciation**

```json
{
  "admin": "addr_safro1g30jss4zxz5ra7efux45etpcmw7vk36vhqm47w",
  "token_denom": "usaf",
  "contribution_amount": "1000000",
  "round_frequency": 604800,
  "beneficiaries": [
    "addr_safro1g30jss4zxz5ra7efux45etpcmw7vk36vhqm47w",
    "addr_safro1djg2upp4enfm57wxc3h4ary8mddn36js33484h",
    "addr_safro12yx0z58fu69xgdq4kr4s4qsqgjpl8hf8lunm4w"
  ],
  "late_penalty": "50000",
  "protocol_fees": "10000",
  "arbitrator": "addr_safro1g30jss4zxz5ra7efux45etpcmw7vk36vhqm47w",
  "time_guards": 86400
}
```

## ⚠️ Important Notes

- **Gardez les phrases mnémoniques** dans un endroit sûr
- **Vérifiez les soldes** avant de lancer les tests
- **Utilisez le script automatique** pour éviter les erreurs manuelles
- **Tous les wallets** doivent être dans le même keyring (`test`)

## 🔍 Troubleshooting

### **Wallet non trouvé**

```bash
safrochaind keys list --keyring-backend test | grep <WALLET_NAME>
```

### **Solde insuffisant**

Vérifiez que chaque wallet a au moins 10,000,000 usaf

### **Erreur de transaction**

Vérifiez que le wallet a suffisamment de fonds pour les frais de gas
