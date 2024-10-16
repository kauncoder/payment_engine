use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};

pub type AmountFloatType = f32;

#[derive(Deserialize, Serialize, Debug)]
pub struct Transaction {
    #[serde(rename = "type")]
    txn_type: String,
    client: u16,
    tx: u32,
    #[serde(default = "default_resource")]
    amount: Option<AmountFloatType>,
}

fn default_resource() -> Option<AmountFloatType> {
    None
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Client {
    pub client: u16,
    pub available: AmountFloatType,
    pub held: AmountFloatType,
    pub total: AmountFloatType,
    pub locked: bool,
}

pub type TxnAmountMap = HashMap<u32, (String, AmountFloatType)>;
enum AmountType {
    Available,
    Held,
    Total,
}

pub type ClientMap = HashMap<u16, Client>;

impl Transaction {
    //read the
    pub fn process(
        &self,
        client_map: &mut ClientMap,
        txn_map: &mut TxnAmountMap,
    ) -> Result<(), Box<dyn Error>> {
        let txn = self.txn_type.as_str();
        match txn {
            "deposit" => {
                //add txn to txn_map,
                txn_map.insert(self.tx, ("deposit".to_owned(), self.amount.unwrap()));
                self.deposit(client_map)?;
            }
            "withdrawal" => {
                //ignore withdrawal with insufficient funds; dont throw error
                if let Ok(_) = self.withdrawal(client_map) {
                    //add txn to txn_map only if it was approved
                    txn_map.insert(self.tx, ("withdrawal".to_owned(), self.amount.unwrap()));
                }
            }
            "dispute" => {
                //ignore error due to
                let _ = self.dispute(client_map, txn_map);
            }
            "resolve" => {
                self.resolve(client_map, txn_map)?;
            }
            "chargeback" => {
                self.chargeback(client_map, txn_map)?;
            }
            _ => return Err(format!("Error: Bad txn type.").into()),
        }
        Ok(())
    }

    fn get_client(id: u16, client_map: &mut ClientMap) -> &mut Client {
        //get id if it exists, else create new one that is empty
        let default_client = Client {
            client: id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        };
        let client = client_map.entry(id).or_insert(default_client);
        client
    }

    fn get_txn_info(
        txn_id: u32,
        txn_map: &mut TxnAmountMap,
    ) -> Result<(String, AmountFloatType), Box<dyn Error>> {
        match txn_map.get(&txn_id) {
            Some((s, txn_amount)) => Ok((s.clone(), *txn_amount)),
            None => {
                return Err(format!("Error: Amount not found. Wrong txn.").into());
            }
        }
    }

    fn deposit(&self, client_map: &mut ClientMap) -> Result<(), Box<dyn Error>> {
        //get client id and call client to change fnuds
        //increase total and available

        let amount = self.amount.unwrap();
        let client = Self::get_client(self.client, client_map);
        //if account is frozen then ignore txn
        if client.locked == true {
            return Ok(());
        }
        client.increase(AmountType::Total, amount);
        client.increase(AmountType::Available, amount);
        Ok(())
    }

    fn withdrawal(&self, client_map: &mut ClientMap) -> Result<(), Box<dyn Error>> {
        //decrease total and available
        let amount = self.amount.unwrap();

        let client = Self::get_client(self.client, client_map);
        //if account is frozen then ignore txn
        if client.locked == true {
            return Ok(());
        }
        //ignore error if withdrawal amount is more than client amounts
        client.decrease(AmountType::Total, amount)?;
        client.decrease(AmountType::Available, amount)?;
        Ok(())
    }

    fn dispute(
        &self,
        client_map: &mut ClientMap,
        txn_map: &mut TxnAmountMap,
    ) -> Result<(), Box<dyn Error>> {
        //read the txn that was diputed and get amount
        //get the amount from previous txn if it exists, else ignore rest
        if let Ok((_, amount)) = Self::get_txn_info(self.tx, txn_map) {
            //decrease available, increase held amount
            let client = Self::get_client(self.client, client_map);
            //if account is frozen then ignore txn
            if client.locked == true {
                return Ok(());
            }
            client.increase(AmountType::Held, amount);
            client.decrease(AmountType::Available, amount)?;
            //add txn to the txn map
            txn_map.insert(self.tx, ("dispute".to_owned(), amount));
        }
        Ok(())
    }

    fn resolve(
        &self,
        client_map: &mut ClientMap,
        txn_map: &mut TxnAmountMap,
    ) -> Result<(), Box<dyn Error>> {
        //read the txn that was resolved and get amount
        if let Ok((txn_type, amount)) = Self::get_txn_info(self.tx, txn_map) {
            if txn_type == "dispute" {
                //decrease held amount, increase available amount
                let client = Self::get_client(self.client, client_map);
                //if account is frozen then ignore txn
                if client.locked == true {
                    return Ok(());
                }
                client.decrease(AmountType::Held, amount)?;
                client.increase(AmountType::Available, amount);
                //remove txn from dispute list
                txn_map.remove(&self.tx);
            }
        }

        Ok(())
    }

    fn chargeback(
        &self,
        client_map: &mut ClientMap,
        txn_map: &mut TxnAmountMap,
    ) -> Result<(), Box<dyn Error>> {
        //read the txn that was resolved and get amount
        if let Ok((txn_type, amount)) = Self::get_txn_info(self.tx, txn_map) {
            if txn_type == "dispute" {
                //decrease held amount
                let client = Self::get_client(self.client, client_map);
                client.locked = true;
                client.decrease(AmountType::Held, amount)?;
            }
        }
        Ok(())
    }
}

impl Client {
    //should have different functions to change values

    fn increase(&mut self, amount_type: AmountType, amount: AmountFloatType) {
        match amount_type {
            AmountType::Available => self.available = self.available + amount,
            AmountType::Held => self.held = self.held + amount,
            AmountType::Total => self.total = self.total + amount,
        }
    }

    fn decrease(
        &mut self,
        amount_type: AmountType,
        amount: AmountFloatType,
    ) -> Result<(), Box<dyn Error>> {
        match amount_type {
            AmountType::Available => {
                if self.available >= amount {
                    self.available = self.available - amount;
                    return Ok(());
                }
            }
            AmountType::Held => {
                if self.held >= amount {
                    self.held = self.held - amount;
                    return Ok(());
                }
            }
            AmountType::Total => {
                if self.total >= amount {
                    self.total = self.total - amount;
                    return Ok(());
                }
            }
        }
        Err(format!("Error: Txn not processed due to insufficient funds.").into())
    }
}
