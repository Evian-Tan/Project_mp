use std::fmt::format;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;

#[derive(Clone)]
struct Account{
    id : usize,
    balance: Arc<Mutex<f64>>,
}
impl  Account {
    fn new(id: usize, balance: f64) -> Self{
        Account { id: id, balance: Arc::new(Mutex::new(balance)), }
    }
}

fn safe_transfer(
    tid: usize,
    from: &Account,
    to: &Account,
    amount: f64,
    timeout: Duration,
) -> Result<(), String>{
    let start = Instant::now();

    //Force requirement: Lock order start with smaller id (T1:lock 3 wait 5 T2:lock 5 wait 3--> lock 3 wait 5-->just race condition)
    let (first,second) = if from.id < to.id {
        (from,to)
    } else {
        (to,from)
    };

    let (mut first_lock, mut second_lock) = loop {
        //First lock
        let first_lock = match first.balance.try_lock() {
            Ok(lock) => lock,
            Err(_) =>{
                //timeout limit
                if Instant::now().duration_since(start) > timeout{
                    println!("Thread{} :Lock Account {} timeout." ,tid,first.id);
                    return Err(format!("Thread{} :Lock Account {} timeout." ,tid,first.id));
                }
                println!("Thread{}: Lock Account {} failed. Retrying...", tid, first.id);
                thread::sleep(Duration::from_millis(40));
                continue;
            }
        };
        //Second lock failed-->release the first and start again
        let second_lock = match second.balance.try_lock() {
            Ok(lock) => lock,
            Err(_) =>{
                drop(first_lock);
                println!("Thread{}: Lock Account {} failed. Release Account{} and retry." ,tid,second.id,first.id);
                thread::sleep(Duration::from_millis(10));
                continue;
            }
        };

        //Acquire the two locks and exit the loop
        break(first_lock,second_lock);

    };
    if *first_lock >= amount {
        *first_lock -= amount;
        *second_lock += amount;
        Ok(())
    } else{
        Err(format!("Error! No sufficient funds in Account {}",first.id))
    }
}
fn main() {
    thread::sleep(Duration::from_secs(3));
    let accounts: Vec<Account> = (0..5)
    .map(|id| Account::new(id, 5500.0))
    .collect();

    let mut handles = vec![];
    for tid in 0..300 {
        let accounts = accounts.clone();
        handles.push(thread::spawn(move || {
            let mut rng = rand::rng();
            let from_index = rng.random_range(0..accounts.len());
            //no same accounts
            let to_index = loop{
                let index = rng.random_range(0..accounts.len());
                if index != from_index {
                    break index;
                }
            };
            //fill in transfer details
            let from = &accounts[from_index];
            let to = &accounts[to_index];
            let amount = rng.random_range(100..5000) as f64 / 100.0;
            let timeout = Duration::from_secs(2);

            let mut retry = 0;
            while retry < 3 {
                match safe_transfer(tid,from, to, amount, timeout){
                    Ok(_) => {
                        println!("Thread{}: Transaction confirmed! (From {} to {}, Amount ${})", tid,from.id,to.id,amount);
                        break;
                    }
                    Err(e) => {
                        println!("Thread{}: Transaction failed! Reason: {})", tid,e);
                        retry += 1;
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
            if retry >= 3{
                println!("Thread{}: Transaction failed after 3 tries! (From {} to {})", tid,from.id,to.id);
            }
        }));
    }
    for handle in handles{
        handle.join().unwrap();
    }
    for account in &accounts{
        println!("Account{} balance: {:.2}", account.id, *account.balance.lock().unwrap());
    }
}    