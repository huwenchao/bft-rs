use bft::*;
use crossbeam::crossbeam_channel::{unbounded, Sender};
use env_logger::Builder;
use env_logger::Target as LogTarget;
use log::LevelFilter;
use rand::{thread_rng, Rng};

use crate::*;

use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

const MAX_TEST_HEIGHT: usize = 10000;

fn is_success(result: Vec<Target>) -> bool {
    let mut result_hashmap: HashMap<Target, u8> = HashMap::new();
    for ii in result.into_iter() {
        let counter = result_hashmap.entry(ii).or_insert(0);
        *counter += 1;
    }
    for (_, count) in result_hashmap.into_iter() {
        if count >= 3 {
            return true;
        }
    }
    false
}

#[cfg(feature = "verify_req")]
#[test]
fn test_verify() {
    Builder::new()
        .default_format_timestamp_nanos(true)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} - {} - {}",
                thread::current().name().clone().unwrap(),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Trace)
        .target(LogTarget::Stdout)
        .init();
    
    let (send_node_0, recv_node_0) = start_process(vec![0]);
    let (send_node_1, recv_node_1) = start_process(vec![1]);
    let (send_node_2, recv_node_2) = start_process(vec![2]);
    let (send_node_3, recv_node_3) = start_process(vec![3]);
    let (send_result, recv_result) = unbounded();

    // simulate the message from executor when executed genesis block
    transmit_genesis(
        send_node_0.clone(),
        send_node_1.clone(),
        send_node_2.clone(),
        send_node_3.clone(),
    );

    // this is the thread of node 0
    let send_0 = send_node_0.clone();
    let send_1 = send_node_1.clone();
    let send_2 = send_node_2.clone();
    let send_3 = send_node_3.clone();
    let send_r = send_result.clone();
    let node_0 = Arc::new(Mutex::new(Node::new()));
    let node_0_clone = node_0.clone();

    let thread_0 = thread::Builder::new()
        .name("node_0".to_string())
        .spawn(move || loop {
            if let Ok(recv) = recv_node_0.recv() {
                node_0_clone.lock().unwrap().handle_message(
                    recv,
                    send_1.clone(),
                    send_2.clone(),
                    send_3.clone(),
                    send_r.clone(),
                );
            }

            if node_0_clone.lock().unwrap().height == MAX_TEST_HEIGHT {
                ::std::process::exit(0);
            }
        })
        .unwrap();

    // this is the thread of node 1
    let send_0 = send_node_0.clone();
    let send_1 = send_node_1.clone();
    let send_2 = send_node_2.clone();
    let send_3 = send_node_3.clone();
    let send_r = send_result.clone();
    let node_1 = Arc::new(Mutex::new(Node::new()));
    let node_1_clone = node_1.clone();

    let thread_1 = thread::Builder::new()
        .name("node_1".to_string())
        .spawn(move || loop {
            if let Ok(recv) = recv_node_1.recv() {
                node_1_clone.lock().unwrap().handle_message(
                    recv,
                    send_0.clone(),
                    send_2.clone(),
                    send_3.clone(),
                    send_r.clone(),
                );
            }

            if node_1_clone.lock().unwrap().height == MAX_TEST_HEIGHT {
                ::std::process::exit(0);
            }
        })
        .unwrap();

    // this is the thread of node 2
    let send_0 = send_node_0.clone();
    let send_1 = send_node_1.clone();
    let send_2 = send_node_2.clone();
    let send_3 = send_node_3.clone();
    let send_r = send_result.clone();
    let node_2 = Arc::new(Mutex::new(Node::new()));
    let node_2_clone = node_2.clone();

    let thread_2 = thread::Builder::new()
        .name("node_2".to_string())
        .spawn(move || loop {
            if let Ok(recv) = recv_node_2.recv() {
                node_2_clone.lock().unwrap().handle_message(
                    recv,
                    send_1.clone(),
                    send_0.clone(),
                    send_3.clone(),
                    send_r.clone(),
                );
            }

            if node_2_clone.lock().unwrap().height == MAX_TEST_HEIGHT {
                ::std::process::exit(0);
            }
        })
        .unwrap();

    // this is the thread of node 3
    let send_0 = send_node_0.clone();
    let send_1 = send_node_1.clone();
    let send_2 = send_node_2.clone();
    let send_3 = send_node_3.clone();
    let send_r = send_result.clone();
    let node_3 = Arc::new(Mutex::new(Node::new()));
    let node_3_clone = node_3.clone();

    let thread_3 = thread::Builder::new()
        .name("node_3".to_string())
        .spawn(move || loop {
            if let Ok(recv) = recv_node_3.recv() {
                node_3_clone.lock().unwrap().handle_message(
                    recv,
                    send_1.clone(),
                    send_2.clone(),
                    send_0.clone(),
                    send_r.clone(),
                );
            }

            if node_3_clone.lock().unwrap().height == MAX_TEST_HEIGHT {
                ::std::process::exit(0);
            }
        })
        .unwrap();

    let thread_commit = thread::Builder::new()
        .name("commit_thread".to_string())
        .spawn(move || {
            let mut chain_height: usize = 2;
            let mut result: Vec<Target> = Vec::new();
            let mut node_0_height = 0;
            let mut node_1_height = 0;
            let mut node_2_height = 0;
            let mut node_3_height = 0;
            let mut now = Instant::now();
            let mut height_result: HashMap<usize, Target> = HashMap::new();

            loop {
                if let Ok(recv) = recv_result.recv() {
                    if recv.clone().address == vec![0] {
                        node_0_height = recv.height + 1;
                    } else if recv.clone().address == vec![1] {
                        node_1_height = recv.height + 1;
                    } else if recv.clone().address == vec![2] {
                        node_2_height = recv.height + 1;
                    } else if recv.clone().address == vec![3] {
                        node_3_height = recv.height + 1;
                    }

                    if height_result.contains_key(&recv.height) {
                        if &recv.proposal != height_result.get(&recv.height).unwrap() {
                            println!("Fork!!!");
                            ::std::process::exit(1);
                        }
                    } else {
                        height_result.insert(recv.height, recv.proposal.clone());
                    }

                    let proposal = generate_proposal();

                    sender[recv.address[0].clone() as usize]
                        .send(BftMsg::Status(Status {
                            height: recv.height.clone(),
                            interval: None,
                            authority_list: generate_auth_list(),
                        }))
                        .unwrap();
                        
                    sender[recv.address[0].clone() as usize]
                        .send(BftMsg::Feed(Feed {
                            height: recv.height.clone() + 1,
                            proposal,
                        }))
                        .unwrap();
                }
                if result.clone().len() >= 3 {
                    if is_success(result.clone()) {
                        println!(
                            "Consensus success at height {:?}, with {:?}",
                            chain_height,
                            Instant::now() - now
                        );
                        result.clear();
                        chain_height += 1;
                        now = Instant::now();
                    } else {
                        ::std::process::exit(1);
                    }
                }
            }
        })
        .unwrap();
}