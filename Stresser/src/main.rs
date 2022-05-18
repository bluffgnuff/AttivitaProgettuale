use std::env;
use log::{debug, info};
use std::{thread, time};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    env_logger::init();
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    let trigger_answer = env::var("TRIGGER_ANSWER").unwrap_or("trigger-answer".to_string());
    let group = env::var("GROUP").unwrap_or("default".to_string());
    // let command = env::var("COMMAND").unwrap_or("../JavaGenericFunctionWithFlag/build/install/JavaGenericFunctionWithFlag/bin/JavaGenericFunctionWithFlag".to_string());
    // let command = env::var("COMMAND").unwrap_or("../JavaMySQLGenericFunction/build/install/JavaMySQLGenericFunction/bin/JavaMySQLGenericFunction".to_string());
    // let command = env::var("COMMAND").unwrap_or("../JavaMongoGenericFunction/build/install/JavaMongoGenericFunction/bin/JavaMongoGenericFunction".to_string());
    let command = env::var("COMMAND").unwrap_or("../JavaCouchGenericFunction/build/install/JavaCouchGenericFunction/bin/JavaCouchGenericFunction".to_string());
    // let command = env::var("COMMAND").unwrap_or("../GenericFunctionWithFlag/target/debug/GenericFunctionWithFlag".to_string());
    // let command = env::var("COMMAND").unwrap_or("../MongoGenericFunction/target/debug/MongoGenericFunction".to_string());
    // let command = env::var("COMMAND").unwrap_or("../MySQLGenericFunction/target/debug/MySQLGenericFunction".to_string());
    // let command = env::var("COMMAND").unwrap_or("../CouchGenericFunction/target/debug/CouchGenericFunction".to_string());
    let args = env::var("COMMAND-ARGS").unwrap_or("--operation Read --table Customers --firstname Mario --lastname Rossi".to_string());
    // let args = env::var("COMMAND_ARGS").unwrap_or("--db-type Mongo --operation Create --table Customers --firstname Giuseppe --lastname Rossi".to_string());
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

    let nc = nats::connect(nats_server).unwrap();
    let nc_clone = nc.clone();
    let sub_command = nc_clone.queue_subscribe(trigger_answer.as_str(), group.as_str()).unwrap();
    debug!("Client | Sub to command topic {:?}", sub_command);

    let mut x = sleep;
    thread::spawn( move ||{
        debug!("Client | start publishing to topic:{}", trigger_command);

        while x >= minsleep {
            for i in 1..batchsize {
                //  Message building
                let req_id = format!("{}-{}", x, i);
                let id_args = format!("{} --id {}",args, req_id);
                let mex = format!("{} {}", command, id_args);
                debug!("Message Built {}", mex);

                //  Header Bulding
                let mut headers = nats::HeaderMap::new();
                headers.append("id", &req_id);
                debug!("Header Built id: {}", headers.get("id").unwrap());

                //  Send the request
                nc.publish_with_reply_or_headers(&trigger_command, None, Option::Some(&headers), mex).unwrap();
                debug!("Request sent");

                let curr_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards");

                info!("[REQUEST_TIME] ID: {}, time: {:?}", req_id, curr_time);

                // Sleep thread
                thread::sleep(time::Duration::from_micros(x));
            }
            x = x / 2;
        }
    });

    //  Waits for the answer from the invokers
    let mut y = sleep;
    while y >= minsleep {
        for i in 1..batchsize {
            //  Receive the response
            let response = sub_command.next().unwrap();
            let answer = String::from_utf8_lossy(&response.data).to_string();
            let received_header = response.headers.unwrap();
            let req_id = received_header.get("id").unwrap();
            debug!("Received answer: {}", answer);

            //  Print
            let curr_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");

            info!("[RESPONSE_TIME] ID: {}, time: {:?}", req_id, curr_time);
        }
        y = y / 2;
    }
}