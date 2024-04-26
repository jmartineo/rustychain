use cli::Cli;

mod tx;
mod cli;
mod block;
mod errors;
mod wallet;
mod blockchain;
mod transaction;



fn main() {
    let cli = Cli::new();
    if let Ok(mut cli) = cli {
        cli.run().unwrap();
    } else {
        println!("Error creating CLI");
    }


}
