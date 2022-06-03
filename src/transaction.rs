use crate::transaction::Transaction::*;

#[derive(Eq, Hash, PartialEq, Debug, Copy, Clone)]
pub struct ClientId(pub u16);

impl ClientId {
    pub const fn new(id: u16) -> ClientId {
        ClientId(id)
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Copy, Clone)]
pub struct TransactionId(u32);

impl TransactionId {
    pub const fn new(id: u32) -> TransactionId {
        TransactionId(id)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TransactionData {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
}

// TODO: we should not use be using floating points to represent currency
pub type MoneyType = f64;

#[derive(Debug)]
pub enum Transaction {
    Deposit {
        data: TransactionData,
        amount: MoneyType,
    },
    Withdrawal {
        data: TransactionData,
        amount: MoneyType,
    },
    Dispute {
        data: TransactionData,
    },
    Resolve {
        data: TransactionData,
    },
    Chargeback {
        data: TransactionData,
    },
}

impl Transaction {
    pub fn client_id(&self) -> ClientId {
        self.data().client_id
    }

    fn data(&self) -> &TransactionData {
        match self {
            Deposit { ref data, .. } => data,
            Withdrawal { ref data, .. } => data,
            Dispute { ref data } => data,
            Resolve { ref data } => data,
            Chargeback { ref data } => data,
        }
    }
}
