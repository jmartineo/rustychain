use clap::{arg, Command};

use crate::blockchain::Blockchain;
use crate::errors::Result;
use crate::transaction::Transaction;
use crate::wallet::Wallets;

pub struct Cli {}

impl Cli {
    pub fn new() -> Result<Cli> {
        Ok(Cli {})
    }

    pub fn run(&mut self) -> Result<()> {
        let matches = Command::new("rustychain")
            .version("0.1")
            .author("jms.martinho@campus.fct.unl.pt")
            .about("a simple blockchain implementation in Rust")
            .subcommand(Command::new("printchain").about("Prints the blockchain"))
            .subcommand(Command::new("getbalance").about("Get the balance of an address")
                .arg(arg!(<ADDRESS>).required(true).index(1)))
            .subcommand(Command::new("create").about("Create a new blockchain")
                .arg(arg!(<ADDRESS>).required(true).index(1)))
            .subcommand(Command::new("send").about("Send an amount to an address")
                .arg(arg!(<FROM>).required(true).index(1))
                .arg(arg!(<TO>).required(true).index(2))
                .arg(arg!(<AMOUNT>).required(true).index(3)))
            .subcommand(Command::new("createwallet").about("Create a new wallet"))
            .subcommand(Command::new("listaddresses").about("List all addresses"))
            .subcommand(Command::new("getwallet").about("Get a wallet")
                .arg(arg!(<ADDRESS>).required(true).index(1)))
            .subcommand(Command::new("listwallets").about("List all wallets"))

            .get_matches();

        if let Some(matches) = matches.subcommand_matches("create") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let address = String::from(address);
                Cli::cmd_create_blockchain(&address)?;
            }
        }

        if let Some(matches) = matches.subcommand_matches("getbalance") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let address = String::from(address);
                Cli::cmd_get_balance(&address)?;
            }
        }

        if let Some(matches) = matches.subcommand_matches("send") {
            if let Some(from) = matches.get_one::<String>("FROM") {
                if let Some(to) = matches.get_one::<String>("TO") {
                    if let Some(amount) = matches.get_one::<String>("AMOUNT") {
                        let from = String::from(from);
                        let to = String::from(to);
                        let amount = amount.parse::<f32>()?;
                        Cli::cmd_send(&from, &to, amount)?;
                    }
                }
            }
        }

        if matches.subcommand_matches("createwallet").is_some() {
            let mut wallets = Wallets::new();
            let address = wallets.create_wallet();
            println!("Wallet created with address: {}", address);
        }

        if matches.subcommand_matches("listaddresses").is_some() {
            let wallets = Wallets::new();
            for address in wallets.get_addresses() {
                println!("{}", address);
            }
        }

        if let Some(matches) = matches.subcommand_matches("getwallet") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let address = String::from(address);
                let wallets = Wallets::new();
                if let Some(wallet) = wallets.get_wallet(&address) {
                    println!("{:#?}", wallet);
                } else {
                    println!("Wallet not found");
                }
            }
        }

        if matches.subcommand_matches("listwallets").is_some() {
            let wallets = Wallets::new();
            for wallet in wallets.get_wallets().values() {
                println!("{:#?}", wallet);
            }
        }

        if matches.subcommand_matches("printchain").is_some() {
            Cli::cmd_print_chain()?;
        }



        Ok(())
    }

    fn cmd_print_chain() -> Result<()> {
        let bc = Blockchain::new()?;
        for b in bc.iter() {
            println!("{:#?}", b);
        }
        Ok(())
    }

    fn cmd_get_balance(address: &str) -> Result<()> {
        let bc = Blockchain::new()?;
        let utxos = bc.find_utxo(address);
        let mut balance = 0.0;
        for out in utxos {
            balance += out.1.get_value();
        }
        println!("Balance of {}: {}", address, balance);
        Ok(())
    }

    fn cmd_create_blockchain(address: &str) -> Result<()> {
        Blockchain::create_blockchain(address.to_owned())?;
        println!("Blockchain created");
        Ok(())
    }

    fn cmd_send(from: &str, to: &str, amount: f32) -> Result<()> {
        let bc = Blockchain::new()?;
        let tx = Transaction::new_utxo(from, to, amount, &bc)?;
        let mut cli = Cli {};
        cli.cmd_add_block(vec![tx])?;
        println!("Transaction sent");
        Ok(())
    }

    fn cmd_add_block(&mut self, txs: Vec<Transaction>) -> Result<()> {
        let mut bc = Blockchain::new()?;
        bc.add_block(txs)?;
        Ok(())
    }
}