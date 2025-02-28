# CS 3502 Project 1: Multi-Threaded Banking System with IPC

This project implements a multithreaded banking transaction simulation system based on Rust (Project A) and inter-process communication pipeline (Project B), with core functions including:
## Overview
1. **Project A**: A multithreaded banking transaction simulation system implemented in four stages: thread creation, resource protection, deadlock demonstration, and resolution.
2. **Project B**: Implementation of pipe-based inter-process communication (IPC), demonstrating data transmission between independent processes.

## Functional Features
| Module            | Function                                                                                  |
|-------------------|-------------------------------------------------------------------------------------------|
| Thread Management | Create 10+ threads to process transactions concurrently and access to Mutex variable      |
| Deadlock Handling | DFS detects resource wait cycles, timeout retries, and lock ordering to prevent deadlocks |
| IPC Communication | Named pipe transmission between two processes                                             |

## Environmental Requirements
- **Operating System**: Linux (Recommended Ubuntu 22.04 LTS)
- **Rust toolchain**: Rust (1.85.0)
- **Editor**: VS Code with rust-analyzer extension (Optional)
- **Dependency Library**:
  - Install system dependencies: `sudo apt install build-essential`
  - Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

- **Quick Start**:
 1. Clone Repository:
     ```bash
     git clone https://github.com/Evian-Tan/Project_mp.git
     cd Project_mp
     ```
2. Build Project:
     ```bash
     cargo build --release
     ```
3. Run Demos:
    - **Project A: Banking Simulation**
      - Phase 1 & 2: `cargo run --bin test_threads_mutex`
      - Phase 3: `cargo run --bin test_deadlock_detect`
      - Phase 4: `cargo run --bin test_deadlock_unlock`
    - **Project B: IPC Pipeline**
      - Terminal 1 (write): `cargo run --bin write`
      - Terminal 2 (read): `cargo run --bin read`
