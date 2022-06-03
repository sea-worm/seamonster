use serde::{Deserialize, Serialize, Serializer};

use crate::{
    Chargeback, ClientId, Deposit, Dispute, MoneyType, Resolve, Transaction, TransactionData,
    TransactionId, Withdrawal,
};

/**
Different CSV data types to keep the IO formats separate from the main application types.
*/

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CSVTransactionKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct CSVTransaction {
    #[serde(rename = "type")]
    type_: CSVTransactionKind,
    client: u16,
    tx: u32,
    amount: Option<MoneyType>,
}

#[derive(Debug, Serialize)]
pub struct CSVSummaryRecord {
    pub client: u16,
    pub available: CSVF64LimitedPrec,
    pub held: CSVF64LimitedPrec,
    pub total: CSVF64LimitedPrec,
    pub locked: bool,
}

#[derive(Debug)]
pub struct CSVF64LimitedPrec(f64);

impl From<f64> for CSVF64LimitedPrec {
    fn from(f: f64) -> Self {
        Self(f)
    }
}

impl serde::Serialize for CSVF64LimitedPrec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:.4}", self.0))
    }
}

impl CSVTransaction {
    pub fn to_transaction(&self) -> Result<Transaction, String> {
        let data = TransactionData {
            client_id: ClientId::new(self.client),
            tx_id: TransactionId::new(self.tx),
        };
        match (self.type_, self.amount) {
            (CSVTransactionKind::Deposit, Some(amount)) => Ok(Deposit { data, amount }),
            (CSVTransactionKind::Withdrawal, Some(amount)) => Ok(Withdrawal { data, amount }),
            (CSVTransactionKind::Dispute, _) => Ok(Dispute { data }),
            (CSVTransactionKind::Resolve, _) => Ok(Resolve { data }),
            (CSVTransactionKind::Chargeback, _) => Ok(Chargeback { data }),
            // super-naive error-handling
            _ => Err(format!("Invalid transaction: {:?}", self)),
        }
    }
}
