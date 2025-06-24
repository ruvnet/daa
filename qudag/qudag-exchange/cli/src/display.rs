//! Display utilities for QuDAG Exchange CLI

use anyhow::Result;
use colored::Colorize;
use qudag_exchange_core::{Ledger, Transaction, TransactionType};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Show network statistics
pub async fn show_stats(ledger: Arc<RwLock<Ledger>>) -> Result<()> {
    let ledger = ledger.read().await;
    let stats = ledger.stats();

    println!("{}", "═══════════════════════════════════════".bright_blue());
    println!("{}", " QuDAG Exchange Network Statistics".bright_white().bold());
    println!("{}", "═══════════════════════════════════════".bright_blue());
    
    println!("Epoch:               {}", stats.epoch.to_string().bright_green());
    println!("Total Supply:        {} rUv", stats.total_supply.to_string().bright_green());
    println!("Active Wallets:      {}", stats.wallet_count.to_string().bright_cyan());
    println!("Pending Txs:         {}", stats.pending_txs.to_string().yellow());
    println!("Confirmed Txs:       {}", stats.confirmed_txs.to_string().bright_green());
    
    println!("{}", "═══════════════════════════════════════".bright_blue());
    
    Ok(())
}

/// Display transaction details
pub fn show_transaction(tx: &Transaction) {
    println!("{}", "═══════════════════════════════════════".bright_blue());
    println!("{}", " Transaction Details".bright_white().bold());
    println!("{}", "═══════════════════════════════════════".bright_blue());
    
    println!("ID:          {}", tx.id.bright_cyan());
    println!("Timestamp:   {}", format_timestamp(tx.timestamp));
    println!("Fee:         {}", format!("{}", tx.fee).bright_yellow());
    
    match &tx.tx_type {
        TransactionType::Transfer { from, to, amount } => {
            println!("Type:        {}", "Transfer".bright_green());
            println!("From:        {}", from.bright_blue());
            println!("To:          {}", to.bright_blue());
            println!("Amount:      {}", format!("{}", amount).bright_green());
        }
        TransactionType::Mint { to, contribution } => {
            println!("Type:        {}", "Mint".bright_green());
            println!("Beneficiary: {}", to.bright_blue());
            println!("Agent:       {}", contribution.agent_id.bright_cyan());
            println!("Amount:      {}", format!("{}", contribution.total_value()).bright_green());
            println!("Verified:    {}", if contribution.verified { "Yes".green() } else { "No".red() });
        }
        TransactionType::Burn { from, amount } => {
            println!("Type:        {}", "Burn".bright_red());
            println!("From:        {}", from.bright_blue());
            println!("Amount:      {}", format!("{}", amount).bright_red());
        }
        TransactionType::FeeDistribution { amount, recipients } => {
            println!("Type:        {}", "Fee Distribution".bright_yellow());
            println!("Total:       {}", format!("{}", amount).bright_green());
            println!("Recipients:  {}", recipients.len());
            for (addr, share) in recipients {
                println!("  {} → {}%", addr.bright_blue(), share);
            }
        }
        TransactionType::Execute { contract, gas_limit, .. } => {
            println!("Type:        {}", "Execute".bright_magenta());
            println!("Contract:    {}", contract.bright_blue());
            println!("Gas Limit:   {}", format!("{}", gas_limit).bright_yellow());
        }
    }
    
    println!("Signed:      {}", if tx.signature.is_some() { "Yes".green() } else { "No".red() });
    
    println!("{}", "═══════════════════════════════════════".bright_blue());
}

/// Format timestamp for display
fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, Utc};
    
    let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| Utc::now());
    
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Progress bar for long operations
pub fn create_progress_bar(total: u64, message: &str) -> indicatif::ProgressBar {
    use indicatif::{ProgressBar, ProgressStyle};
    
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());
    pb
}

// Add chrono dependency for timestamp formatting
use chrono;