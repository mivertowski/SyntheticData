# Fraud Detection ML

Train machine learning models for financial fraud detection.

## Overview

DataSynth generates labeled datasets for supervised fraud detection:

- 20+ fraud patterns with full labels
- Graph representations for GNN models
- Realistic data distributions
- Configurable fraud rates and types

## Configuration

```yaml
global:
  seed: 42
  industry: financial_services
  start_date: 2024-01-01
  period_months: 12

transactions:
  target_count: 100000

fraud:
  enabled: true
  fraud_rate: 0.02                   # 2% fraud rate

  types:
    split_transaction: 0.20
    duplicate_payment: 0.15
    fictitious_transaction: 0.15
    ghost_employee: 0.10
    kickback_scheme: 0.10
    revenue_manipulation: 0.10
    expense_capitalization: 0.10
    unauthorized_discount: 0.10

anomaly_injection:
  enabled: true
  total_rate: 0.02
  generate_labels: true

  categories:
    fraud: 1.0                       # Focus on fraud only

graph_export:
  enabled: true
  formats:
    - pytorch_geometric

  split:
    train: 0.7
    val: 0.15
    test: 0.15
    stratify: is_fraud

output:
  format: csv
```

## Output Files

### Tabular Data

```
output/
├── transactions/
│   └── journal_entries.csv
├── labels/
│   ├── anomaly_labels.csv
│   └── fraud_labels.csv
└── master_data/
    └── ...
```

### Graph Data

```
output/graphs/transaction_network/pytorch_geometric/
├── node_features.pt
├── edge_index.pt
├── edge_attr.pt
├── labels.pt
├── train_mask.pt
├── val_mask.pt
└── test_mask.pt
```

## ML Pipeline

### 1. Load Data

```python
import pandas as pd
import torch

# Load tabular data
entries = pd.read_csv('output/transactions/journal_entries.csv')
labels = pd.read_csv('output/labels/fraud_labels.csv')

# Merge
data = entries.merge(labels, on='document_id', how='left')
data['is_fraud'] = data['fraud_type'].notna()

print(f"Total entries: {len(data)}")
print(f"Fraud entries: {data['is_fraud'].sum()}")
print(f"Fraud rate: {data['is_fraud'].mean():.2%}")
```

### 2. Feature Engineering

```python
from sklearn.preprocessing import StandardScaler, OneHotEncoder

# Numerical features
numerical_features = [
    'debit_amount', 'credit_amount', 'line_count'
]

# Derived features
data['log_amount'] = np.log1p(data['debit_amount'] + data['credit_amount'])
data['is_round'] = (data['debit_amount'] % 100 == 0).astype(int)
data['is_weekend'] = pd.to_datetime(data['posting_date']).dt.dayofweek >= 5
data['is_month_end'] = pd.to_datetime(data['posting_date']).dt.day >= 28

# Categorical features
categorical_features = ['source', 'business_process', 'company_code']
```

### 3. Train Model (Tabular)

```python
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split
from sklearn.metrics import classification_report, roc_auc_score

# Prepare features
X = data[numerical_features + derived_features]
y = data['is_fraud']

# Split
X_train, X_test, y_train, y_test = train_test_split(
    X, y, test_size=0.2, stratify=y, random_state=42
)

# Train
model = RandomForestClassifier(n_estimators=100, random_state=42)
model.fit(X_train, y_train)

# Evaluate
y_pred = model.predict(X_test)
y_prob = model.predict_proba(X_test)[:, 1]

print(classification_report(y_test, y_pred))
print(f"ROC-AUC: {roc_auc_score(y_test, y_prob):.4f}")
```

### 4. Train GNN Model

```python
import torch
import torch.nn.functional as F
from torch_geometric.nn import GCNConv
from torch_geometric.data import Data

# Load graph data
node_features = torch.load('output/graphs/.../node_features.pt')
edge_index = torch.load('output/graphs/.../edge_index.pt')
labels = torch.load('output/graphs/.../labels.pt')
train_mask = torch.load('output/graphs/.../train_mask.pt')
val_mask = torch.load('output/graphs/.../val_mask.pt')
test_mask = torch.load('output/graphs/.../test_mask.pt')

data = Data(
    x=node_features,
    edge_index=edge_index,
    y=labels,
    train_mask=train_mask,
    val_mask=val_mask,
    test_mask=test_mask,
)

# Define GNN
class FraudGNN(torch.nn.Module):
    def __init__(self, num_features, hidden_channels):
        super().__init__()
        self.conv1 = GCNConv(num_features, hidden_channels)
        self.conv2 = GCNConv(hidden_channels, hidden_channels)
        self.linear = torch.nn.Linear(hidden_channels, 2)

    def forward(self, x, edge_index):
        x = self.conv1(x, edge_index).relu()
        x = F.dropout(x, p=0.5, training=self.training)
        x = self.conv2(x, edge_index).relu()
        x = self.linear(x)
        return x

# Train
model = FraudGNN(data.num_features, 64)
optimizer = torch.optim.Adam(model.parameters(), lr=0.01)

for epoch in range(200):
    model.train()
    optimizer.zero_grad()
    out = model(data.x, data.edge_index)
    loss = F.cross_entropy(out[data.train_mask], data.y[data.train_mask])
    loss.backward()
    optimizer.step()

    # Validation
    if epoch % 10 == 0:
        model.eval()
        pred = out.argmax(dim=1)
        val_acc = (pred[data.val_mask] == data.y[data.val_mask]).float().mean()
        print(f'Epoch {epoch}: Val Acc: {val_acc:.4f}')
```

## Fraud Types for Training

| Type | Detection Approach | Difficulty |
|------|-------------------|------------|
| Split Transaction | Amount patterns | Easy |
| Duplicate Payment | Similarity matching | Easy |
| Fictitious Transaction | Anomaly detection | Medium |
| Ghost Employee | Entity verification | Medium |
| Kickback Scheme | Relationship analysis | Hard |
| Revenue Manipulation | Trend analysis | Hard |

## Best Practices

### Class Imbalance

```python
from imblearn.over_sampling import SMOTE

# Handle imbalanced classes
smote = SMOTE(random_state=42)
X_resampled, y_resampled = smote.fit_resample(X_train, y_train)
```

### Threshold Tuning

```python
from sklearn.metrics import precision_recall_curve

# Find optimal threshold
precision, recall, thresholds = precision_recall_curve(y_test, y_prob)
f1_scores = 2 * precision * recall / (precision + recall)
optimal_idx = f1_scores.argmax()
optimal_threshold = thresholds[optimal_idx]
```

### Cross-Validation

```python
from sklearn.model_selection import StratifiedKFold

cv = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)
scores = cross_val_score(model, X, y, cv=cv, scoring='roc_auc')
print(f"CV ROC-AUC: {scores.mean():.4f} (+/- {scores.std():.4f})")
```

## See Also

- [Anomaly Injection](../advanced/anomaly-injection.md)
- [Graph Export](../advanced/graph-export.md)
- [Configuration - Compliance](../configuration/compliance.md)
