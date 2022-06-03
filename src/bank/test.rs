#[cfg(test)]
mod tests {
    use crate::bank::{ClientSummary, Engine};
    use crate::transaction::Transaction::*;
    use crate::transaction::*;

    static CLIENT_A: ClientId = ClientId::new(1);

    static TX1: TransactionId = TransactionId::new(1);
    static TX2: TransactionId = TransactionId::new(2);

    fn run(transactions: Vec<Transaction>) -> Engine {
        let mut engine = Engine::create();
        for tx in transactions {
            // ignore invalid transactions for tests
            engine.update(tx).unwrap_or(());
        }
        engine
    }

    fn test_run(transactions: Vec<Transaction>) -> Vec<ClientSummary> {
        let engine = run(transactions);
        Vec::from_iter(engine.iter().map(|(_k, v)| v))
    }

    #[test]
    fn test_deposit_withdraw() {
        let balances = test_run(vec![deposit(2.0, TX1), withdraw(1.0, TX2)]);
        assert_eq!(
            balances,
            vec!(ClientSummary {
                available: 1.0,
                held: 0.0,
                locked: false,
            })
        );
    }

    #[test]
    fn test_deposit_withdraw_empty() {
        let balances = test_run(vec![withdraw(1.0, TX1)]);
        assert_eq!(
            balances,
            vec!(ClientSummary {
                available: 0.0,
                held: 0.0,
                locked: false,
            })
        );
    }

    #[test]
    fn test_deposit_dispute() {
        let balances = test_run(vec![deposit(2.0, TX1), deposit(1.0, TX2), dispute(TX1)]);
        assert_eq!(
            balances,
            vec!(ClientSummary {
                available: 1.0,
                held: 2.0,
                locked: false,
            })
        );
    }

    #[test]
    fn test_withdrawal_dispute_insufficient_balance() {
        let balances = test_run(vec![deposit(2.0, TX1), withdraw(1.0, TX2), dispute(TX1)]);
        assert_eq!(
            balances,
            vec!(ClientSummary {
                available: 1.0,
                held: 0.0,
                locked: false,
            })
        );
    }

    #[test]
    fn test_double_dispute() {
        let balances = test_run(vec![
            deposit(2.0, TX1),
            dispute(TX1),
            deposit(1.0, TX2),
            dispute(TX1),
        ]);
        assert_eq!(
            balances,
            vec!(ClientSummary {
                available: 1.0,
                held: 2.0,
                locked: false,
            })
        );
    }

    #[test]
    fn test_deposit_dispute_resolve() {
        let balances = test_run(vec![
            deposit(2.0, TX1),
            deposit(1.0, TX2),
            dispute(TX1),
            resolve(TX1),
        ]);
        assert_eq!(
            balances,
            vec!(ClientSummary {
                available: 3.0,
                held: 0.0,
                locked: false,
            })
        );
    }

    #[test]
    fn test_deposit_dispute_chargeback() {
        let balances = test_run(vec![
            deposit(2.0, TX1),
            deposit(1.0, TX2),
            dispute(TX1),
            charge_back(TX1),
        ]);
        assert_eq!(
            balances,
            vec!(ClientSummary {
                available: 1.0,
                held: 0.0,
                locked: true,
            })
        );
    }

    #[test]
    fn test_deposit_dispute_double_chargeback() {
        let balances = test_run(vec![
            deposit(2.0, TX1),
            deposit(1.0, TX2),
            dispute(TX1),
            charge_back(TX1),
            charge_back(TX1),
        ]);
        assert_eq!(
            balances,
            vec!(ClientSummary {
                available: 1.0,
                held: 0.0,
                locked: true,
            })
        );
    }

    #[test]
    fn test_deposit_dispute_after_chargeback() {
        let balances = test_run(vec![
            deposit(2.0, TX1),
            deposit(1.0, TX2),
            dispute(TX1),
            charge_back(TX1),
            dispute(TX1),
        ]);
        assert_eq!(
            balances,
            vec!(ClientSummary {
                available: 1.0,
                held: 0.0,
                locked: true,
            })
        );
    }

    #[test]
    fn test_deposit_dispute_after_resolve() {
        let balances = test_run(vec![
            deposit(2.0, TX1),
            deposit(1.0, TX2),
            dispute(TX1),
            resolve(TX1),
            dispute(TX1),
        ]);
        assert_eq!(
            balances,
            vec!(ClientSummary {
                available: 1.0,
                held: 2.0,
                locked: false,
            })
        );
    }

    /**
    A few helpers to conveniently create transactions and allow changes to the system
    without needing to change all tests.
    */

    fn deposit(amount: MoneyType, tx_id: TransactionId) -> Transaction {
        Deposit {
            data: TransactionData {
                client_id: CLIENT_A,
                tx_id,
            },
            amount,
        }
    }

    fn withdraw(amount: MoneyType, tx_id: TransactionId) -> Transaction {
        Withdrawal {
            data: TransactionData {
                client_id: CLIENT_A,
                tx_id,
            },
            amount,
        }
    }

    fn dispute(tx_id: TransactionId) -> Transaction {
        Dispute {
            data: TransactionData {
                client_id: CLIENT_A,
                tx_id,
            },
        }
    }

    fn resolve(tx_id: TransactionId) -> Transaction {
        Resolve {
            data: TransactionData {
                client_id: CLIENT_A,
                tx_id,
            },
        }
    }

    fn charge_back(tx_id: TransactionId) -> Transaction {
        Chargeback {
            data: TransactionData {
                client_id: CLIENT_A,
                tx_id,
            },
        }
    }
}
