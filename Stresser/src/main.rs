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
    // let command = env::var("COMMAND").unwrap_or("../GenericFunctionWithFlag/target/debug/GenericFunctionWithFlag".to_string());
    let command = env::var("COMMAND").unwrap_or("../CouchGenericFunction/target/debug/CouchGenericFunction".to_string());
    // let args = env::var("COMMAND-ARGS").unwrap_or("\"SELECT * FROM Customers WHERE FirstName = 'Mario';\"".to_string());
    // let args = env::var("COMMAND-ARGS").unwrap_or("--operation Create --table Customers --firstname Mario --lastname Rossi".to_string());
    // let args = env::var("COMMAND_ARGS").unwrap_or("--db-type Mongo --operation Create --table Customers --firstname Mario --lastname Rossi".to_string());
    let args = env::var("COMMAND_ARGS").unwrap_or("--operation Read --table Customers --firstname Mario --lastname Rossi".to_string());
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
    let minsleep = env::var("MINSLEEP")
        .unwrap_or("1000".to_string())
        .parse::<u64>()
        .unwrap();

    let nc = nats::connect(&nats_server).unwrap();
    debug!("Client | start publishing to topic:{}", trigger_command);
    let mut x = sleep;
    while x >= minsleep {
        let mut average = 0;
        let mut min = 0;
        let mut max = 0;
        let mut total_duration = 0;

        for i in 1..batchsize {
            let req_id = format!("{}-{}", x, i);
            // Add the req id in the message arguments
            let id_args = format!("{} --id {}",args, req_id);
            let mex = format!("{} {}", command, id_args);

            let start_time = SystemTime::now();
            let resp = nc.request(&trigger_command, mex).unwrap();
            let duration = SystemTime::now().duration_since(start_time).unwrap();

            total_duration = total_duration + duration.as_micros();

            if duration.as_micros() > max{
                max = duration.as_micros();
            }

            if duration.as_micros() < min || min == 0{
                min = duration.as_micros();
            }

            average =  total_duration/(i as u128);

            info!("[RESPONSE_TIME] msg: {}, latency: {} μs",req_id,duration.as_micros());
            debug!("{:?}", String::from_utf8_lossy(&resp.data).to_string());
            thread::sleep(time::Duration::from_micros(x));
        }
        info!("[RESPONSE_TIME_AVERAGE] time sleep: {}, latency: {} μs",x, average);
        info!("[RESPONSE_TIME_MIN] time sleep: {}, latency: {} μs",x, min);
        info!("[RESPONSE_TIME_MAX] time sleep: {}, latency: {} μs",x, max);
         x = x / 2;
    }
}