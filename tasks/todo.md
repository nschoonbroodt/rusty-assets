# High Priority Issues


# Remaining Development Tasks

## Reporting & Analytics (Implementation Details)
- [ ] Automatic spending classification (based on rules, machine learning?)
- [ ] Trial balance report
- [ ] CSV/JSON export for reports
- [ ] Date range validation and defaults (current month, year, YTD, etc.)

### Reporting Database Views
Create SQL views for common reporting queries to optimize performance and avoid code duplication:
- Balance sheet view with account hierarchies
- Income statement view with revenue/expense categorization
- Cash flow view with operating/investing/financing activities
- Trial balance view with current balances by account

### Cash Flow Improvements (Lower Priority)
- [ ] Refine account categorization (e.g., large "Pending" transfers)
- [ ] Better investment activity detection
- [ ] Add beginning/ending cash balance display
- [ ] Enhanced category mapping for more accurate activity classification

## System Improvements (Code Quality & Minor Features)
- [ ] Error handling improvements with user-friendly messages
- [ ] Account archiving/deactivation for closed accounts

---

# Future Ideas (Long Term - To Be Refined)

## Advanced Features
- [ ] Budget goal tracking
- [ ] Automatic loan prediction  
- [ ] Future tax estimation
- [ ] Multi "main currency" support

## Platform Extensions
- [ ] Web API
- [ ] Mobile app
- [ ] Local web app

