use std::env;
use log::debug;

fn main() {
    env_logger::init();
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    // let trigger_args = env::var("TRIGGER-ARGS").unwrap_or("trigger-args".to_string());
    let group = env::var("GROUP").unwrap_or("default".to_string());
    let command = env::var("COMMAND").unwrap_or("../GenericFunction/target/debug/GenericFunction".to_string());
    let args = env::var("COMMAND-ARGS").unwrap_or("Delete Customers Eugenio Rossi Claudio Villa".to_string());

    debug!("Client | start publishing to topic:{}", trigger_command);
    // debug!("Client | start publishing to topic:{}", trigger_args);
    let mex = format!("{} {}", command, args);
    let nc = nats::connect(&nats_server).unwrap();
    nc.publish(&trigger_command, mex);
    // nc.publish(&trigger_args, args);
}