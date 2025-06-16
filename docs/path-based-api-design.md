# Path-Based API Design

**Date**: June 16, 2025  
**Context**: Demo data creation revealed API usability issues. Current APIs require manual ID tracking and verbose hierarchical creation.

## Problem Statement

### Current Account Creation (Painful)
```rust
let assets = account_service.create_account(NewAccount::builder().name("Assets")...).await?;
let current_assets = account_service.create_account(
    NewAccount::builder()
        .parent_id(assets.id)  // Manual ID tracking!
        .name("Current Assets")...
).await?;
```

### Current Transaction Creation (Verbose)
```rust
let checking_account = account_service.get_account_by_path("Assets:Current Assets:Main Checking").await?;
let salary_account = account_service.get_account_by_path("Income:Employment:Salary").await?;
let salary_transaction = TransactionService::create_simple_transaction(
    "Monthly Salary Deposit".to_string(),
    checking_account.id,     // Manual account resolution
    salary_account.id,       // More manual resolution
    Decimal::new(320000, 2), // Verbose amount creation
    previous_month_start,
    None, None,
);
transaction_service.create_transaction(salary_transaction).await?;
```

## Proposed Solution

### Path-Based Account Creation
```rust
#[derive(Builder)]
pub struct NewAccountByPath {
    #[builder(into)]
    full_path: String,  // "Assets:Current Assets:Main Checking"
    account_type: AccountType,
    account_subtype: AccountSubtype,
    #[builder(default)]
    currency: String,
}

// Usage:
account_service.create_account_by_path(
    NewAccountByPath::builder()
        .full_path("Assets:Current Assets:Main Checking")
        .account_type(AccountType::Asset)
        .account_subtype(AccountSubtype::Checking)
        .build()
).await?;
```

### Path-Based Transaction Creation
```rust
#[derive(Builder)]
pub struct NewTransactionByPath {
    description: String,
    #[builder(into)]
    debit_account_path: String,
    #[builder(into)] 
    credit_account_path: String,
    amount: Decimal,
    date: DateTime<Utc>,
    #[builder(default)]
    reference: Option<String>,
}

// Usage:
transaction_service.create_transaction_by_path(
    NewTransactionByPath::builder()
        .description("Monthly Salary Deposit")
        .debit_account("Assets:Current Assets:Main Checking")
        .credit_account("Income:Employment:Salary")
        .amount(Decimal::new(320000, 2))
        .date(previous_month_start)
        .build()
).await?;
```

## Transaction Creation

### Current State
Transaction creation currently requires manual account ID lookups and verbose entry construction:

```rust
// Current approach - verbose and error-prone
let checking_account = account_service.get_account_by_path(&user_id, "Assets:Current Assets:Main Checking").await?;
let salary_account = account_service.get_account_by_path(&user_id, "Income:Salary").await?;

let transaction = NewTransaction::builder()
    .user_id(user_id)
    .description("Monthly salary")
    .date(NaiveDate::from_ymd_opt(2025, 1, 31).unwrap())
    .build();

let transaction_id = transaction_service.create_transaction(&transaction).await?;

// Manual journal entry creation
let entries = vec![
    NewJournalEntry::builder()
        .transaction_id(transaction_id)
        .account_id(checking_account.id)
        .debit_amount(Some(Decimal::from(3500)))
        .credit_amount(None)
        .build(),
    NewJournalEntry::builder()
        .transaction_id(transaction_id)
        .account_id(salary_account.id)
        .debit_amount(None)
        .credit_amount(Some(Decimal::from(3500)))
        .build(),
];

for entry in entries {
    transaction_service.create_journal_entry(&entry).await?;
}
```

### Proposed Path-Based API

#### Builder Pattern with Path-Based Entries
```rust
use bon::Builder;

#[derive(Builder)]
#[builder(on(String, into), on(&str, into)]
pub struct NewTransactionByPath {
    pub user_id: Uuid,
    pub description: String,
    pub date: NaiveDate,
    
    #[builder(default)]
    pub entries: Vec<JournalEntryByPath>,
}

#[derive(Builder)]
#[builder(on(String, into), on(&str, into)]
pub struct JournalEntryByPath {
    pub account_path: String,
    pub amount: Decimal,
    pub entry_type: EntryType, // Debit or Credit
}

#[derive(Debug, Clone)]
pub enum EntryType {
    Debit,
    Credit,
}
```

#### Ergonomic Transaction Creation
```rust
// Simple two-account transaction
let transaction = NewTransactionByPath::builder()
    .user_id(user_id)
    .description("Monthly salary")
    .date(NaiveDate::from_ymd_opt(2025, 1, 31).unwrap())
    .entries(vec![
        JournalEntryByPath::builder()
            .account_path("Assets:Current Assets:Main Checking")
            .amount(Decimal::from(3500))
            .entry_type(EntryType::Debit)
            .build(),
        JournalEntryByPath::builder()
            .account_path("Income:Salary")
            .amount(Decimal::from(3500))
            .entry_type(EntryType::Credit)
            .build(),
    ])
    .build();

let transaction_id = transaction_service.create_transaction_by_path(&transaction).await?;
```

#### Helper Methods for Common Transaction Types
```rust
impl NewTransactionByPath {
    /// Create a simple two-account transaction (debit/credit)
    pub fn simple_transfer(
        user_id: Uuid,
        description: impl Into<String>,
        date: NaiveDate,
        from_account: impl Into<String>,
        to_account: impl Into<String>,
        amount: Decimal,
    ) -> Self {
        Self::builder()
            .user_id(user_id)
            .description(description)
            .date(date)
            .entries(vec![
                JournalEntryByPath::builder()
                    .account_path(to_account)
                    .amount(amount)
                    .entry_type(EntryType::Debit)
                    .build(),
                JournalEntryByPath::builder()
                    .account_path(from_account)
                    .amount(amount)
                    .entry_type(EntryType::Credit)
                    .build(),
            ])
            .build()
    }
    
    /// Create an expense transaction (debit expense, credit asset/liability)
    pub fn expense(
        user_id: Uuid,
        description: impl Into<String>,
        date: NaiveDate,
        expense_account: impl Into<String>,
        payment_account: impl Into<String>,
        amount: Decimal,
    ) -> Self {
        Self::simple_transfer(user_id, description, date, payment_account, expense_account, amount)
    }
    
    /// Create an income transaction (debit asset, credit income)
    pub fn income(
        user_id: Uuid,
        description: impl Into<String>,
        date: NaiveDate,
        income_account: impl Into<String>,
        deposit_account: impl Into<String>,
        amount: Decimal,
    ) -> Self {
        Self::simple_transfer(user_id, description, date, income_account, deposit_account, amount)
    }
}
```

#### Ultra-Concise Transaction Creation
```rust
// Income transaction
let salary = NewTransactionByPath::income(
    user_id,
    "Monthly salary",
    NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
    "Income:Salary",
    "Assets:Current Assets:Main Checking",
    Decimal::from(3500),
);

// Expense transaction
let groceries = NewTransactionByPath::expense(
    user_id,
    "Weekly groceries",
    NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
    "Expenses:Food:Groceries",
    "Assets:Current Assets:Main Checking",
    Decimal::from(120),
);

// More complex multi-entry transaction
let restaurant_with_tip = NewTransactionByPath::builder()
    .user_id(user_id)
    .description("Dinner at restaurant")
    .date(NaiveDate::from_ymd_opt(2025, 1, 20).unwrap())
    .entries(vec![
        JournalEntryByPath::builder()
            .account_path("Expenses:Food:Restaurants")
            .amount(Decimal::from(45))
            .entry_type(EntryType::Debit)
            .build(),
        JournalEntryByPath::builder()
            .account_path("Expenses:Tips")
            .amount(Decimal::from(8))
            .entry_type(EntryType::Debit)
            .build(),
        JournalEntryByPath::builder()
            .account_path("Assets:Current Assets:Main Checking")
            .amount(Decimal::from(53))
            .entry_type(EntryType::Credit)
            .build(),
    ])
    .build();
```

### Implementation Details

#### Service Layer Method
```rust
impl TransactionService {
    pub async fn create_transaction_by_path(
        &self,
        transaction: &NewTransactionByPath,
    ) -> Result<Uuid, CoreError> {
        // 1. Resolve all account paths to IDs (with auto-creation if needed)
        let mut account_ids = HashMap::new();
        for entry in &transaction.entries {
            if !account_ids.contains_key(&entry.account_path) {
                let account = self.account_service
                    .get_or_create_account_by_path(&transaction.user_id, &entry.account_path)
                    .await?;
                account_ids.insert(entry.account_path.clone(), account.id);
            }
        }
        
        // 2. Create the transaction
        let new_transaction = NewTransaction::builder()
            .user_id(transaction.user_id)
            .description(&transaction.description)
            .date(transaction.date)
            .build();
        
        let transaction_id = self.create_transaction(&new_transaction).await?;
        
        // 3. Create journal entries
        for entry in &transaction.entries {
            let account_id = account_ids[&entry.account_path];
            let (debit_amount, credit_amount) = match entry.entry_type {
                EntryType::Debit => (Some(entry.amount), None),
                EntryType::Credit => (None, Some(entry.amount)),
            };
            
            let journal_entry = NewJournalEntry::builder()
                .transaction_id(transaction_id)
                .account_id(account_id)
                .debit_amount(debit_amount)
                .credit_amount(credit_amount)
                .build();
            
            self.create_journal_entry(&journal_entry).await?;
        }
        
        Ok(transaction_id)
    }
}
```

### Benefits

1. **Ergonomic**: Intuitive account references using familiar paths
2. **Maintainable**: No manual ID lookups or verbose entry construction
3. **Flexible**: Supports both simple and complex multi-entry transactions
4. **Auto-creation**: Missing accounts are created automatically
5. **Type-safe**: Builder pattern ensures correct construction
6. **Concise**: Helper methods for common transaction patterns
7. **Readable**: Transaction intent is clear from the code

### Migration Strategy

1. Implement `NewTransactionByPath` and `create_transaction_by_path` alongside existing APIs
2. Update demo code to use the new path-based API
3. Add integration tests to validate functionality
4. Gradually migrate existing code to use path-based API
5. Eventually deprecate ID-based transaction creation APIs

This approach makes transaction creation as intuitive as writing accounting journal entries, while maintaining the flexibility to handle complex multi-entry transactions when needed.
```
