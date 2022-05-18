use nats::{Connection};
use std::process::{Command, Stdio};
use std::time::{ SystemTime};
use log::{debug, info};
use std::{env};
use std::io::{BufRead, BufReader};

fn work(command: String) -> String {
    //  Invoking the command
    let child = Command::new("/bin/bash").arg("-c").arg(&command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    debug!("Child launched PID = {}", child.id());

    let mut child_out = BufReader::new(child.stdout.unwrap()).lines();

    //  Return the child's output
    // let res = child_out.next().unwrap().unwrap();
    // debug!("Child output: {}", res);
    // return  res;

    // Read function Stdout. It could present a new line char so it's necessary to read all the output until the end
    let mut res= String::new();
    let mut first = true;
    loop{
        match child_out.next(){
            Some(val) => {
                if first {
                    res = format!("{}", val.unwrap());
                    first = false;
                }else {
                    res = format!("{}, {}",res, val.unwrap());
                }
            },
            None => {
                break;
            },
        }
    }
    // //  Return the child's output
    res
}

fn server (nc: Connection, trigger_command: String, trigger_answer: String, group: String){
    let mut n_reqs = 0;
    let mut total_latency = 0;
    let mut max = 0;
    let mut min = 0;

    let sub_command = nc.queue_subscribe(trigger_command.as_str(), group.as_str()).unwrap();
    debug!("Sub to command topic {:?}", sub_command);

    loop {
        //  Consuming message
        let mex = sub_command.next().unwrap();
        let received_header = mex.headers.unwrap();
        let req_id = received_header.get("id").unwrap();
        let command =  String::from_utf8_lossy(&mex.data).to_string();
        debug!("New request: {} received command: {}", req_id, command);

        //  Launch operation
        n_reqs = n_reqs +1;
        let start_time = SystemTime::now();
        let child_out = work(command);
        let work_latency = SystemTime::now().duration_since(start_time).unwrap();
        debug!("Child ouput: {}", child_out);

        //  Answer to stresser
        let mut headers = nats::HeaderMap::new();
        headers.append("id", req_id);
        nc.publish_with_reply_or_headers(&trigger_answer, None, Option::Some(&headers), child_out).unwrap();
        debug!("Answer to {} sent", req_id);

        //  Update general stats work
        total_latency = total_latency + work_latency.as_micros();
        if work_latency.as_micros() > max{
            max = work_latency.as_micros();
        }
        if work_latency.as_micros() < min || min == 0{
            min = work_latency.as_micros();
        }
        let average = total_latency/(n_reqs as u128);

        //  Print Stats
        info!("[WORK_LATENCY] request number {}: latency {} μs", n_reqs, work_latency.as_micros());
        info!("[WORK_AVERAGE_LATENCY] request number {}: average latency {} μs", n_reqs, average);
        info!("[WORK_MIN_LATENCY] request number {}: {} μs", n_reqs, min);
        info!("[WORK_MAX_LATENCY] request number {}: max latency {} μs", n_reqs, max);
    }
}

fn main() {
    env_logger::init();
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    let trigger_answer = env::var("TRIGGER_ANSWER").unwrap_or("trigger-answer".to_string());
    let group = env::var("GROUP").unwrap_or("default".to_string());

    debug!("Starts");

    //  Connection to MOM
    let nc = nats::connect(nats_server.as_str()).unwrap();
    debug!("Connected to NATS {:?} ", nc);
    debug!("Start publishing to topic:{}", trigger_answer);

    server(nc, trigger_command, trigger_answer, group);
}