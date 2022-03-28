use std::env;
use log::{debug, info};
use std::{thread, time};
use std::time::SystemTime;

fn main() {
    env_logger::init();
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    // let trigger_args = env::var("TRIGGER-ARGS").unwrap_or("trigger-args".to_string());
    // let group = env::var("GROUP").unwrap_or("default".to_string());
    let command = env::var("COMMAND").unwrap_or("../GenericFunctionWithFlag/target/debug/GenericFunctionWithFlag".to_string());
    // let args = env::var("COMMAND-ARGS").unwrap_or("\"SELECT * FROM Customers WHERE FirstName = 'Mario';\"".to_string());
    // let args = env::var("COMMAND-ARGS").unwrap_or("--operation Create --table Customers --firstname Mario --lastname Rossi".to_string());
    let args = env::var("COMMAND-ARGS").unwrap_or("--db-type MySQL --operation Read --table Customers --firstname Mario --lastname Rossi".to_string());
    // let args = env::var("COMMAND-ARGS").unwrap_or("--operation Update --table Customers --firstname Mario --lastname Rossi --firstname-op Luca --firstname-op Villa".to_string());
    // let args = env::var("COMMAND-ARGS").unwrap_or("--operation Update --table Customers --id --rev --firstname-op Luca --firstname-op Villa".to_string());
    // let args = env::var("COMMAND-ARGS").unwrap_or("--operation Delete --table Customers --id ".to_string());
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
            let req_id = format!("{}.{}", x, i);
            // Add the req id in the message arguments
            let id_args = format!("{} --id {}",args, req_id);
            let mex = format!("{} {}", command, id_args);

            let send_time = SystemTime::now();
            let resp = nc.request(&trigger_command, mex).unwrap();
            info!("[RESPONSE_TIME] msg: {}, time: {}",req_id,SystemTime::now().duration_since(send_time).unwrap().as_micros());
            debug!("{:?}", String::from_utf8_lossy(&resp.data).to_string());
            thread::sleep(time::Duration::from_micros(x));
        }
         x = x / 2;
    }
}