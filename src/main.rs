use std::thread;
use std::time::Duration;
use std::sync::{Arc,Mutex};

struct Account {
    id: usize,
    login: bool,
    transaction: bool,
}

impl Account {
    fn new(id: usize) -> Self{
        Account {
            id,
            login: false,
            transaction: false,
        }
    }
}

struct Bank{
    accounts: Vec<Account>,
}

impl Bank{
    fn new() -> Self{
        Bank {
            accounts: Vec::new(),
        }
    }
    fn register(&mut self, account:Account){
        self.accounts.push(account);
    }
}

#[test]
fn test_mp_transactions(){
    let bank = Arc::new(Mutex::new(Bank::new()));
    let mut handles = vec![];

    for i in 1..=10 {
        let bank_clone = Arc::clone(&bank);
        let handle = thread::spawn(move || {
            let mut account = Account::new(i);
            
            account.login = true;
            println!("Thread {}: Account {} logged in.", i, account.id);
            thread::sleep(Duration::from_millis(50));
            account.transaction = true;
            println!("Thread {}: Transaction submitted for account {}.", i, account.id);

            if account.transaction {
                account.login = false;
                println!("Thread {}: Transaction completed. ID:{} Logged out.", i, account.id);
            }
            else{
                println!("Thread {}: Operation failed for account {}.", i, account.id);
            }
            {
                let mut bank = bank_clone.lock().unwrap();
                bank.register(account);
            }
        });
        handles.push(handle)
    }
    for handle in handles{
        handle.join().unwrap();
    }
    let bank = bank.lock().unwrap();
    println!("Total accounts registered in bank: {}", bank.accounts.len());
    for (account) in bank.accounts.iter() {
        println!(
            "Account {}, login: {}, transaction: {}",
            account.id, account.login, account.transaction
        );
    }
}