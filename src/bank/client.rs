use std::collections::HashMap;

use crate::transaction::Transaction::*;
use crate::transaction::{MoneyType, Transaction, TransactionData, TransactionId};

#[derive(Debug, PartialEq)]
enum TransactionState {
    Normal,
    Disputed,
    ChargedBack,
}

#[derive(Debug)]
struct TransactionBlock {
    state: TransactionState,
    tx: Transaction,
}

impl TransactionBlock {
    fn new(tx: Transaction) -> TransactionBlock {
        TransactionBlock {
            state: TransactionState::Normal,
            tx,
        }
    }
}

#[derive(Debug)]
pub struct Client {
    pub(in crate::bank) available: MoneyType,
    pub(in crate::bank) held: MoneyType,
    pub(in crate::bank) locked: bool,
    tx_chain: HashMap<TransactionId, TransactionBlock>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ClientSummary {
    pub available: MoneyType,
    pub held: MoneyType,
    pub locked: bool,
}

#[derive(Debug)]
pub enum ClientError {
    NotEnoughBalance,
    Locked,
    InvalidTransactionId,
    InvalidDispute,
}

pub type CResult = Result<(), ClientError>;

impl Client {
    pub(crate) fn new(amount: f64) -> Client {
        Client {
            available: amount,
            held: 0.0,
            locked: false,
            // For now we only store deposit TXs here in the client account (as they are the only ones which can be disputed)
            tx_chain: HashMap::new(),
        }
    }

    pub fn update(&mut self, tx: Transaction) -> CResult {
        if self.locked {
            return Err(ClientError::Locked);
        }
        match tx {
            Deposit { ref data, amount } => {
                self.deposit(amount)?;
                // In this implementation only deposits can be disputed -> we only need to store this transaction kind for now
                self.tx_chain.insert(data.tx_id, TransactionBlock::new(tx));
            }
            Withdrawal { amount, .. } => {
                self.withdraw(amount)?;
            }
            Dispute { ref data } => self.dispute(&data)?,
            Resolve { ref data } => self.resolve(&data)?,
            Chargeback { ref data } => self.chargeback(&data)?,
        };
        Ok(())
    }

    fn deposit(&mut self, amount: MoneyType) -> CResult {
        // TODO: we should use a decimal library for money (as this could either overflow or at least run into high imprecision)
        self.available += amount;
        Ok(())
    }

    fn withdraw(&mut self, amount: MoneyType) -> CResult {
        let new_balance = self.available - amount;
        if new_balance < 0.0 {
            return Err(ClientError::NotEnoughBalance);
        }
        self.available = new_balance;
        Ok(())
    }

    fn dispute(&mut self, tx_data: &TransactionData) -> CResult {
        let (tx_block, dispute_amount) = self.get_dispute_tx(&tx_data, TransactionState::Normal)?;
        tx_block.state = TransactionState::Disputed;
        self.mark_held(dispute_amount)?;
        Ok(())
    }

    fn resolve(&mut self, tx_data: &TransactionData) -> CResult {
        let (tx_block, dispute_amount) =
            self.get_dispute_tx(&tx_data, TransactionState::Disputed)?;
        tx_block.state = TransactionState::Normal;
        self.mark_held(-dispute_amount)?;
        Ok(())
    }

    fn chargeback(&mut self, tx_data: &TransactionData) -> CResult {
        let (tx_block, dispute_amount) =
            self.get_dispute_tx(&tx_data, TransactionState::Disputed)?;
        tx_block.state = TransactionState::ChargedBack;
        self.held -= dispute_amount;
        self.locked = true;
        return Ok(());
    }

    fn get_dispute_tx(
        &mut self,
        tx_data: &TransactionData,
        tx_state: TransactionState,
    ) -> Result<(&mut TransactionBlock, MoneyType), ClientError> {
        let tx_block = self
            .tx_chain
            .get_mut(&tx_data.tx_id)
            .ok_or_else(|| ClientError::InvalidTransactionId)?;
        let dispute_amount = match tx_block {
            TransactionBlock {
                state,
                tx: Deposit { amount, .. },
            } if *state == tx_state => Ok(*amount),
            _ => Err(ClientError::InvalidDispute),
        }?;
        Ok((tx_block, dispute_amount))
    }

    fn mark_held(&mut self, amount: MoneyType) -> CResult {
        let new_balance = self.available - amount;
        if new_balance < 0.0 {
            return Err(ClientError::NotEnoughBalance);
        }
        self.available = new_balance;
        self.held += amount;
        Ok(())
    }

    pub(crate) fn summary(&self) -> ClientSummary {
        ClientSummary {
            held: self.held,
            available: self.available,
            locked: self.locked,
        }
    }
}
