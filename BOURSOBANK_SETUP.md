# BoursoBank Import Setup Commands

This guide provides the complete list of commands to set up your database for importing BoursoBank CSV transactions.

## Prerequisites

1. Initialize the database:
```bash
cargo run -- db init
```

## Step 1: Create a User

First, create your user account:

```bash
# Create your user
cargo run -- users add --name "you" --display-name "Your Full Name"

# Save the UUID that gets displayed
export USER_ID="<uuid-from-output>"

# Or get the UUID later with:
cargo run -- users get you
```

## Step 2: Create Account Structure

The BoursoBank importer expects the following account hierarchy. Create these accounts:

### Main Bank Account
```bash
# Your main BoursoBank account (adjust name as needed)
cargo run -- accounts create
# When prompted:
# - Name: BoursoBank
# - Type: Asset
# - Subtype: Checking
# - Parent: Assets:Current Assets (if it exists, otherwise create the hierarchy)
```

### Required Account Structure for Import

The importer will try to categorize transactions to these accounts. Create them using the interactive account creation:

#### Asset Accounts
```bash
# Main asset hierarchy (if not already created)
cargo run -- accounts create # Assets (Asset, OtherAsset)
cargo run -- accounts create # Current Assets (Asset, Cash) - parent: Assets
cargo run -- accounts create # BoursoBank (Asset, Checking) - parent: Assets:Current Assets
cargo run -- accounts create # Credit Card (Asset, CreditCard) - parent: Assets:Current Assets
cargo run -- accounts create # Savings (Asset, Savings) - parent: Assets
cargo run -- accounts create # Insurance (Asset, OtherAsset) - parent: Assets:Savings
```

#### Income Accounts
```bash
cargo run -- accounts create # Income (Income, Salary)
cargo run -- accounts create # Salary (Income, Salary) - parent: Income
cargo run -- accounts create # Investment (Income, Investment) - parent: Income
cargo run -- accounts create # Other (Income, Investment) - parent: Income
```

#### Expense Accounts
```bash
# Main expense hierarchy
cargo run -- accounts create # Expenses (Expense, OtherExpense)

# Food expenses
cargo run -- accounts create # Food (Expense, OtherExpense) - parent: Expenses
cargo run -- accounts create # Groceries (Expense, OtherExpense) - parent: Expenses:Food
cargo run -- accounts create # Restaurants (Expense, OtherExpense) - parent: Expenses:Food

# Transportation expenses
cargo run -- accounts create # Transportation (Expense, OtherExpense) - parent: Expenses
cargo run -- accounts create # Fuel (Expense, OtherExpense) - parent: Expenses:Transportation
cargo run -- accounts create # Parking (Expense, OtherExpense) - parent: Expenses:Transportation
cargo run -- accounts create # Tolls (Expense, OtherExpense) - parent: Expenses:Transportation
cargo run -- accounts create # Taxi (Expense, OtherExpense) - parent: Expenses:Transportation
cargo run -- accounts create # Public (Expense, OtherExpense) - parent: Expenses:Transportation
cargo run -- accounts create # Other (Expense, OtherExpense) - parent: Expenses:Transportation

# Personal expenses
cargo run -- accounts create # Personal (Expense, OtherExpense) - parent: Expenses
cargo run -- accounts create # Clothing (Expense, OtherExpense) - parent: Expenses:Personal
cargo run -- accounts create # Sports (Expense, OtherExpense) - parent: Expenses:Personal
cargo run -- accounts create # Other (Expense, OtherExpense) - parent: Expenses:Personal

# Home expenses
cargo run -- accounts create # Home (Expense, OtherExpense) - parent: Expenses
cargo run -- accounts create # Maintenance (Expense, OtherExpense) - parent: Expenses:Home

# Housing expenses
cargo run -- accounts create # Housing (Expense, OtherExpense) - parent: Expenses
cargo run -- accounts create # Mortgage (Expense, OtherExpense) - parent: Expenses:Housing

# Travel expenses
cargo run -- accounts create # Travel (Expense, OtherExpense) - parent: Expenses
cargo run -- accounts create # Accommodation (Expense, OtherExpense) - parent: Expenses:Travel

# Utilities expenses
cargo run -- accounts create # Utilities (Expense, OtherExpense) - parent: Expenses
cargo run -- accounts create # Subscriptions (Expense, OtherExpense) - parent: Expenses:Utilities

# Financial expenses
cargo run -- accounts create # Financial (Expense, OtherExpense) - parent: Expenses
cargo run -- accounts create # Fees (Expense, OtherExpense) - parent: Expenses:Financial

# Entertainment expenses
cargo run -- accounts create # Entertainment (Expense, OtherExpense) - parent: Expenses

# Uncategorized (fallback)
cargo run -- accounts create # Uncategorized (Expense, OtherExpense) - parent: Expenses
```

## Step 3: Verify Setup

Check that everything is created correctly:

```bash
# List all users
cargo run -- users list

# Show account tree
cargo run -- accounts tree

# List all accounts
cargo run -- accounts list
```

## Step 4: Import Your Data

Now you can import your BoursoBank CSV file:

```bash
cargo run -- import boursobank \\
  --file "perso/BoursoBank/courant/2025/export-operations-2025-04.csv" \\
  --account "Assets:Current Assets:BoursoBank" \\
  --user-id $USER_ID
```

## Step 5: Verify Import

Check your data:

```bash
# Generate balance sheet
cargo run -- reports balance-sheet

# Generate income statement  
cargo run -- reports income-statement --user-id $USER_ID
```

## Notes

- The importer will automatically categorize transactions based on BoursoBank's category system
- If an expected account doesn't exist, the importer will fall back to "Income:Other" or "Expenses:Uncategorized"
- You can adjust account names and hierarchy as needed for your preference
- All accounts support the full path format (e.g., "Assets:Current Assets:BoursoBank")

## Account Path Reference

Here are the exact paths the importer expects:

**Income Paths:**
- `Income:Salary` (for "Virements reçus")
- `Income:Investment` (for "Revenus d'épargne")
- `Income:Other` (fallback)

**Expense Paths:**
- `Expenses:Food:Groceries` (Alimentation)
- `Expenses:Food:Restaurants` (Restaurants, bars, discothèques…)
- `Expenses:Personal:Clothing` (Vêtements et accessoires)
- `Expenses:Personal:Sports` (Equipements sportifs et artistiques)
- `Expenses:Personal:Other` (other Vie quotidienne)
- `Expenses:Home:Maintenance` (Bricolage et jardinage)
- `Expenses:Transportation:Fuel` (Carburant)
- `Expenses:Transportation:Parking` (Parking)
- `Expenses:Transportation:Tolls` (Péages)
- `Expenses:Transportation:Taxi` (Taxis)
- `Expenses:Transportation:Public` (Transports quotidiens)
- `Expenses:Transportation:Other` (other Auto & Moto)
- `Expenses:Travel:Accommodation` (Hébergement)
- `Expenses:Travel` (other Voyages & Transports)
- `Expenses:Entertainment` (Loisirs et sorties)
- `Expenses:Utilities:Subscriptions` (Abonnements & téléphonie)
- `Expenses:Housing:Mortgage` (Logement)
- `Expenses:Financial:Fees` (Services financiers & professionnels)
- `Expenses:Uncategorized` (fallback)

**Asset Paths:**
- `Assets:Current Assets:Credit Card` (Mouvements internes débiteurs)
- `Assets:Savings:Insurance` (Dépenses d'épargne)
