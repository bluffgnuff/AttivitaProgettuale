use std::env;
use log::{debug, info};
use std::{thread, time};
use std::time::SystemTime;

fn main() {
    env_logger::init();
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    let trigger_answer = env::var("TRIGGER_ANSWER").unwrap_or("trigger-answer".to_string());
    let group = env::var("GROUP").unwrap_or("default".to_string());
    // let command = env::var("COMMAND").unwrap_or("../GenericFunctionWithFlag/target/debug/GenericFunctionWithFlag".to_string());
    // let command = env::var("COMMAND").unwrap_or("../MongoGenericFunction/target/debug/MongoGenericFunction".to_string());
    let command = env::var("COMMAND").unwrap_or("../CouchGenericFunction/target/debug/CouchGenericFunction".to_string());
    // let args = env::var("COMMAND-ARGS").unwrap_or("\"SELECT * FROM Customers WHERE FirstName = 'Mario';\"".to_string());
    let args = env::var("COMMAND-ARGS").unwrap_or("--operation Read --table Customers --firstname Mario --lastname Rossi".to_string());
    // let args = env::var("COMMAND_ARGS").unwrap_or("--db-type MySQL --operation Read --table Customers --firstname Mario --lastname Rossi".to_string());
    // let args = env::var("COMMAND_ARGS").unwrap_or("--operation Create --table Customers --firstname Mario --lastname Rossi".to_string());
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
    let sub_command = nc.queue_subscribe(trigger_answer.as_str(), group.as_str()).unwrap();
    debug!("Invoker | Sub to command topic {:?}", sub_command);

    let ack =1;
    let mut x = sleep;
    while x >= minsleep {
        let mut average = 0;
        let mut min = 0;
        let mut max = 0;
        let mut total_latency = 0;
        let mut average_conf = 0;
        let mut min_conf = 0;
        let mut max_conf = 0;
        let mut total_latency_conf = 0;

        for i in 1..batchsize {
            //  Message building
            let req_id = format!("{}-{}", x, i);
            let id_args = format!("{} --id {}",args, req_id);
            let mex = format!("{} {}", command, id_args);

            let start_time = SystemTime::now();

            //  Send the request
            let conf = nc.request(&trigger_command, mex).unwrap();
            let conf_latency = SystemTime::now().duration_since(start_time).unwrap();
            info!("[MESSAGE_LATENCY] msg: {}, latency: {} μs",req_id,conf_latency.as_micros());
            total_latency_conf = total_latency_conf + conf_latency.as_micros();

            //  Receive the response
            let resp = sub_command.next().unwrap();
            resp.respond(ack.to_string());
            let latency = SystemTime::now().duration_since(start_time).unwrap();
            total_latency = total_latency + latency.as_micros();

            //  Update general stats request
            if conf_latency.as_micros() > max{
                max_conf = conf_latency.as_micros();
            }
            if conf_latency.as_micros() < min || min == 0{
                min_conf = conf_latency.as_micros();
            }
            average_conf = total_latency_conf/(i as u128);

            //  Update general stats response
            if latency.as_micros() > max{
                max = latency.as_micros();
            }
            if latency.as_micros() < min || min == 0{
                min = latency.as_micros();
            }
            average = total_latency/(i as u128);

            info!("[RESPONSE_LATENCY] msg: {}, latency: {} μs",req_id,latency.as_micros());
            debug!("{:?}", String::from_utf8_lossy(&resp.data).to_string());
            thread::sleep(time::Duration::from_micros(x));
        }
        //  Print Stats
        info!("[MESSAGE_AVERAGE_LATENCY] time sleep: {}, latency: {} μs",x, average_conf);
        info!("[MESSAGE_MIN_LATENCY] time sleep: {}, latency: {} μs",x, min_conf);
        info!("[MESSAGE_MAX_LATENCY] time sleep: {}, latency: {} μs",x, max_conf);

        info!("[RESPONSE_AVERAGE_LATENCY] time sleep: {}, latency: {} μs",x, average);
        info!("[RESPONSE_MIN_LATENCY] time sleep: {}, latency: {} μs",x, min);
        info!("[RESPONSE_MAX_LATENCY] time sleep: {}, latency: {} μs",x, max);
         x = x / 2;
    }
}