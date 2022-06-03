use std::collections::HashMap;

use crate::bank::client::{CResult, Client};
use crate::bank::ClientSummary;
use crate::transaction::{ClientId, Transaction};

pub struct Engine {
    clients: HashMap<ClientId, Client>,
}

impl Engine {
    pub fn create() -> Engine {
        Engine {
            // super-naive map of clients
            clients: HashMap::new(),
        }
    }

    pub fn update(&mut self, tx: Transaction) -> CResult {
        let client = self.find_or_create_client(tx.client_id());
        client.update(tx)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ClientId, ClientSummary)> + '_ {
        self.clients.iter().map(|(k, v)| (k, v.summary()))
    }

    fn find_or_create_client(&mut self, client_id: ClientId) -> &mut Client {
        self.clients.entry(client_id).or_insert(Client::new(0.0))
    }
}
