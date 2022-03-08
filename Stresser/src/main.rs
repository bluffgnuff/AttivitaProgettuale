use std::env;
use log::{debug, info};
use std::{thread, time};
use std::time::SystemTime;

fn main() {
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    // let trigger_args = env::var("TRIGGER-ARGS").unwrap_or("trigger-args".to_string());
    // let group = env::var("GROUP").unwrap_or("default".to_string());
    let command = env::var("COMMAND").unwrap_or("../GenericFunction/target/debug/GenericFunction".to_string());
    let args = env::var("COMMAND-ARGS").unwrap_or("Read Customers Luca Rossi".to_string());
    let sleep = env::var("SLEEP")
        .unwrap_or("1000000".to_string())
        .parse::<u64>()
        .unwrap();
    let batchsize = env::var("BATCHSIZE")
        .unwrap_or("100".to_string())
        .parse::<u64>()
        .unwrap();
    let chainlenght = env::var("CHAINLENGHT")
        .unwrap_or("3".to_string())
        .parse::<u64>()
        .unwrap();
    let minsleep = env::var("MINSLEEP")
        .unwrap_or("1000".to_string())
        .parse::<u64>()
        .unwrap();
    debug!("Client | start publishing to topic:{}", trigger_command);

    let nc = nats::connect(&nats_server).unwrap();
    let mut x = sleep;

    while x >= minsleep {
        for i in 1..batchsize {
            let mex = format!("{} {}", command, args);
            nc.publish(&trigger_command, mex);
            // nc.publish(&trigger_args, args);
            let id = format!("{}.{}", x, i);
            info!(
                "STARTCHAIN msg:{}, start:{:?}",
                id,
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("time went backwards")
                    .as_nanos()
            );
            thread::sleep(time::Duration::from_micros(x));
        }
        x = x / 2;
    }
}
