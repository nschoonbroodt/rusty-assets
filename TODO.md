# Actual Todo

## Example data: include account tree with deeper nesting

For example:
Assets -> Bank1 -> Cash
Assets -> Bank1 -> Saving
Assets -> Bank1 -> Brokerage
Assets -> Bank1 -> Brokerage --> AAPL
Assets -> Bank1 -> Brokerage --> MSFT

## Actually display account balance

The "assets-cli accounts balance" command says "Balance" calculation coming soon

## ownership command show user id. Please show the username or displayname or whatever is available

## account create don't create right now

Next Steps Available:
🔄 Balance Calculation: Implement actual balance calculation from journal entries
🔄 Interactive Account Creation: Build the full account creation workflow
🔄 Enhanced Tree View: Add support for deeper account hierarchies
🔄 User-Friendly Ownership Display: Show user names instead of UUIDs

# Done

## ✅ Database access problem - SOLVED

~~is this due to the initial connexion only?~~
**SOLVED**: The issue was DNS resolution of `localhost` on Windows taking ~21 seconds per connection.
Changed DATABASE_URL from `localhost` to `127.0.0.1` which reduced command time from 21.8s to 0.8s (96.5% improvement).

## ✅ use Euro as main currency - DONE

~~All example uses usd as main value. Can we use euro instead?~~

**COMPLETED**: Updated all examples, demos, documentation, and database defaults to use EUR (€) instead of USD ($):

- ✅ Database migration: Changed default currency from 'USD' to 'EUR'
- ✅ Demo transactions: Updated all amounts to use € symbol
- ✅ README examples: Changed from $ to € in double-entry examples
- ✅ Account balance display: Shows € symbol for average cost
- ✅ All monetary examples: €3,000 salary, €150 groceries, €2,500 stock purchase, etc.
