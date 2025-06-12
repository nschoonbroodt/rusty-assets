# Actual Todo

## check sqlx macro usage

Even with the instruction, copilot did use the macros in some place. Need to check that.

## Automated Price Feeds

Implement automated price feeds from financial APIs for real-time price updates instead of manual entry. This would allow the system to automatically fetch current market prices for tracked assets (stocks, ETFs, crypto) from sources like Yahoo Finance, Alpha Vantage, or similar APIs.

## Add github actions

## Real-world transaction import (CSV, QIF, OFX)

Support importing transactions from common financial data formats (CSV, QIF, OFX) to make onboarding and data migration easier.

## Web or GUI interface

Develop a web or graphical user interface as an alternative to the CLI for broader accessibility and ease of use.

## Reporting: balance sheets, income statements, net worth tracking

Implement reporting features to generate balance sheets, income statements, and net worth tracking for comprehensive financial analysis.

## Create reporting command

- general balance
- income vs expense
- performance of assets
- allocation
- net worth summary

All of these for entire familly and by user

## import bank statement

- csv or other
- automatic spending classification (based on rule, machine learning?)

## Error handling improvements

Add proper error handling and user-friendly error messages throughout the CLI, especially for database operations and user input validation.

## Transaction search and filtering

Add commands to search/filter transactions by date range, account, amount, or description to make the system more useful with real data.

## Backup and restore functionality

Add commands to export/import the entire database for backup purposes, especially important for personal finance data.

## Account archiving/deactivation

Add ability to archive old accounts without deleting historical data (useful for closed bank accounts, sold investments, etc.).

# Medium term

## Create a terminal user interface with ratatui

## Security

Does the database needs to be crypted? Do we need auth? Row based access?

## Auto update prices

such as share prices, crypto

# Long term - to be sorted

## UI using a rust framework

## UI using tauri

## possible web api

## possible mobile app

## possible web app (but local)

## budget goal tracking

## automatic loan prediction?

## future tax estimation

## multi "main currency" support?
