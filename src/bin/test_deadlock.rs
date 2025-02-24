use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::{HashMap,HashSet};
use rand::Rng;

struct Account{
    id: usize,
    //Only one process modify the balance at a time
    balance: Mutex<f64>,
}

impl Account {
    fn new(id: usize, balance: f64) -> Self{
        Account {
            id,
            balance: Mutex::new(balance),
        }
    }
}

struct DeadlockDetector{
    lock_state: Mutex<HashMap<usize, (HashSet<usize>,HashSet<usize>)>>
}
impl DeadlockDetector{
    fn new() -> Self {
        DeadlockDetector{
            lock_state: Mutex::new(HashMap::new()),
        }
    }
    fn update_state(&self, tid: usize, hold: HashSet<usize>, wait: HashSet<usize>){
        let mut lock_state = self.lock_state.lock().unwrap();
        lock_state.insert(tid, (hold, wait));
    }  

    fn detect_deadlock(&self) -> Option<Vec<usize>> {  
        let mut lock_state = self.lock_state.lock().unwrap();
        let mut graph = HashMap::new();
        //find those who hold others' wanted -->deadlock (hold & wait)
        for(&tid,(_,wait)) in lock_state.iter(){
            let mut waiting_threads = HashSet::new();
            for &resource in wait{
                for (&other_tid, (other_hold,_)) in lock_state.iter(){
                    if other_hold.contains(&resource) {
                        waiting_threads.insert(other_tid);
                    }
                }
            }       
            if !waiting_threads.is_empty(){
                graph.insert(tid,waiting_threads);
            }
        }
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        fn dfs(
            node: usize,
            graph: &HashMap<usize, HashSet<usize>>,
            visited: &mut HashSet<usize>,
            stack: &mut HashSet<usize>,
        ) -> bool {
            //loop detect
            if stack.contains(&node) {
                return true;
            }
            if visited.contains(&node) {
                return false;
            }

            visited.insert(node);
            stack.insert(node);
            //neighbors is the waited thread(those who take its wanted thread)
            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    if dfs(*neighbor, graph, visited, stack) {
                        return true;
                    }
                }
            }
            //no cycle, clean visiting stack and return false
            stack.remove(&node);
            false
        }    

        for &node in graph.keys() {
            let mut local_stack = HashSet::new();
            if dfs(node, &graph, &mut visited, &mut local_stack) {
                return Some(local_stack.into_iter().collect());
            }
        }
        None
    }
}

fn transfer(
    from: &Account,
    to: &Account,
    amount: f64,
    detector: &DeadlockDetector,
    tid: usize,
) {
    let first_lock = from.balance.try_lock();
    let mut hold = HashSet::new();
    let mut wait = HashSet::new();

    match first_lock{
        Ok(_) => {
            hold.insert(from.id);
            match to.balance.try_lock(){
                Ok(_) => {
                    hold.insert(to.id);
                    if from.balance >= amount{
                        from.balance -= amount;
                        to.balance += amount;
                        println!("Thread{}: Transaction confirmed! (From {} to {}, Amount ${})", tid,from.id,to.id,amount);
                    }
                },
                Err(_) => {
                    wait.insert(to.id);
                }
            }    
        },
        Err(_) => {
            wait.insert(from.id);
        }
    }
    detector.update_state(tid, hold, wait);
}
fn main() {
    let accounts: Vec<Arc<Account>> = (0..5)
        .map(|i| Arc::new(Account::new(i, 1000.0)))
        .collect();

    let detector = Arc::new(DeadlockDetector::new());
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let accounts = accounts.clone();
            let detector = detector.clone();
            thread::spawn(move || {
                let tid = i;
                loop {
                    let mut rng = rand::rng();
                    let from_index = rng.random_range(0..accounts.len());
                    //no same accounts
                    let to_index = loop{
                    let index = rng.random_range(0..accounts.len());
                    if index != from_index {
                    break index;
                }
            };

                    let from = &accounts[from_index];
                    let to = &accounts[to_index];

                    // transfer
                    transfer(from, to, 10.0, &detector, tid);

                    // detect
                    if let Some(cycle) = detector.detect_deadlock() {
                        println!("DEADLOCK DETECTED: Threads in cycle: {:?}", cycle);
                    }

                    thread::sleep(Duration::from_millis(100));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}    