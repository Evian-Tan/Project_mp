use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::{HashMap,HashSet};

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
#[derive(Debug, Clone)]
struct TransferRecord{
    thread_id: usize,
    from: usize,
    to: usize,
    start_time: Instant,
}
#[derive(Clone)]
struct DeadlockDetector{
    //record list of each thread transfer(Create list-->DFS-->loop detect)
    active_transfers: Arc<Mutex<Vec<TransferRecord>>>,
}

impl DeadlockDetector{
    fn new() -> Self{
        DeadlockDetector {
            active_transfers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn start_transfer(&self, tid: usize, from: usize, to: usize){
        let mut records = self.active_transfers.lock().unwrap();
        records.push(TransferRecord{
            thread_id: tid,
            from,
            to,
            start_time: Instant::now(),
        });
    }
    fn end_transfer(&self, tid: usize) {
        let mut records = self.active_transfers.lock().unwrap();
        records.retain(|r| r.thread_id != tid);
    }

    fn detect(&self) -> Option<Vec<usize>>{
        //timeout detect abnormal thread activity
        let timeout = Duration::from_secs(2);
        let now = Instant::now();
        //sort out those timeout records
        let timed_out: Vec<_> = self.active_transfers.lock().unwrap()
            .iter()
            .filter(|r| now.duration_since(r.start_time) > timeout)
            .cloned()
            .collect();

        if timed_out.is_empty(){return None;}

        let mut graph: HashMap<usize, HashSet<usize>> = HashMap::new();
        for r in &timed_out {
            graph.entry(r.from).or_default().insert(r.to);
        }
        let mut visited = HashMap::new();
        for &node in graph.keys() {
            if Self::dfs_detect_cycle(node, &graph, &mut visited) {
                return Some(Self::extract_cycle(node, &graph));
            }
        }
        None
    }
    fn dfs_detect_cycle(
        node: usize,
        graph: &HashMap<usize, HashSet<usize>>,
        visited: &mut HashMap<usize, bool>,
    ) -> bool {
        match visited.get(&node) {
            Some(&true) => return true,    // loops detect
            Some(&false) => return false,  // complete
            None => {
                visited.insert(node, true); // being visited
                for &neighbor in graph.get(&node).unwrap_or(&HashSet::new()) {
                    if Self::dfs_detect_cycle(neighbor, graph, visited) {
                        return true;
                    }
                }
                visited.insert(node, false); 
                false
            }
        }
    }
    fn extract_cycle(start: usize, graph: &HashMap<usize, HashSet<usize>>) -> Vec<usize> {
        let mut cycle = vec![start];
        let mut current = start;
        
        while let Some(next) = graph.get(&current).and_then(|edges| edges.iter().next()) {
            if *next == start {
                cycle.push(*next);
                break;
            }
            cycle.push(*next);
            current = *next;
        }
        cycle
    }
}
fn main() {

    let accounts: Vec<Arc<Account>> = (0..5)
        .map(|id| Arc::new(Account::new(id, 1000.0)))
        .collect();

    let detector = DeadlockDetector::new();
    let mut handles = vec![];

    for tid in 0..10 {
        let accounts = accounts.clone();
        let det = detector.clone();
        
        handles.push(thread::spawn(move || {
            //10 threads deal with 5 accounts(++Deadlock)
            let from_idx = tid % 5;
            let to_idx = (tid + 1) % 5;
            let from = &accounts[from_idx];
            let to = &accounts[to_idx];

            det.start_transfer(tid, from.id, to.id);
            let (first, second) = if tid % 2 == 0 {
                (from, to)
            } else {
                (to, from)
            };
            
            let _lock1 = first.balance.lock().unwrap();
            thread::sleep(Duration::from_millis(100));
            let _lock2 = second.balance.lock().unwrap();
            
            thread::sleep(Duration::from_secs(3));
            det.end_transfer(tid);
        }));
    }
    loop {
        thread::sleep(Duration::from_secs(1));
        if let Some(cycle) = detector.detect() {
            println!("Deadlock detected in cycle: {:?}", cycle);
            break;
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
}