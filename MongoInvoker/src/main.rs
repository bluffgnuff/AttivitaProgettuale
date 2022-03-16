#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use rmps::Serializer;
use serde::Serialize;
use std::{env, str};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{ SystemTime};
use log::{debug, info};
use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::sync::{Client, Database};
use nats::Subscription;
//Usage env parameters --URL {URL} --DB-NAME {DB-NAME} --COMMAND {COMMAND}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
enum Op {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
struct Request {
    op: Op,
    table : String,
    param: HashMap<String, String>,
    param_to_up: Option<HashMap<String, String>>,
}
// Convert the parameter of the the request to a Bson Document;
// Set update=true if the Document is used in an Update operation
fn from_param_to_doc(param : HashMap<String, String>, update: bool) -> Document {
    let mut first = true;
    let mut doc: Document = Default::default();
    //	Split name, val
    for p in param{
         if p.0 == "_id" {
             doc.insert(p.0.as_str(), ObjectId::parse_str(p.1.as_str()).unwrap());
         }
         else {
             doc.insert(p.0.as_str(),p.1.as_str());
        }
    }
    if update{
       doc = doc!( "$set" : doc);
    }
    debug!("Document Created {}", doc);
    doc
}

fn work(conn: &mut Database, command: String) {
    // Invoking the command
    let mut child = Command::new("/bin/bash").arg("-c").arg(&command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    debug!("Invoker | child launched PID = {}", child.id());

    let child_in = child.stdin.as_mut().unwrap();
    let mut child_out = BufReader::new(child.stdout.unwrap()).lines();

    // Receive
    let mut out = child_out.next().unwrap().unwrap();
    out.remove(0);
    out.remove(out.len()-1);
    debug!("Invoker | request cleaned {:?}", out);

    drop(child_out);

    let req_serialized:Vec<u8> = out.split(", ").map(|x| x.parse().unwrap()).collect();
    debug!("Invoker | serialized request {:?}", req_serialized);

    //  Deserialize
    let req: Request = rmp_serde::from_read_ref(&req_serialized).unwrap();
    debug!("Invoker | deserialized request {:?}", req);

    //  Document generation
    let document = from_param_to_doc(req.param, false);
    debug!("Invoker | document to execute: {}", document);

    //  Query execution
    let mut result_serialized  = Vec::new();
    match req.op{
        Op::Create => {
            let query_result =
                match conn.collection(req.table.as_str()).insert_one(document,None){
                    Ok(_) => String::from("Success"),
                    Err(_) =>String::from("Error")
                };

            query_result.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },
        Op::Read => {
            // Send back a Vec<Row> to keep the invoker independent from the data type
            let query_result = conn.collection::<Document>(req.table.as_str()).find(document,None).unwrap();
            let mut query_string :Vec<String>= Vec::new();
            for el in query_result{
                query_string.push(format!("{:?}", el.unwrap()));
            }
            debug!("Invoker | res: {:?}", query_string);

            query_string.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },

        Op::Update => {
            let update_doc = from_param_to_doc(req.param_to_up.unwrap(), true);
            let query_result =
                match conn.collection::<HashMap<String, String>>(req.table.as_str()).update_one(document,update_doc, None){
                Ok(_) => String::from("Success"),
                Err(_) =>String::from("Error")
            };

            query_result.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },
        Op::Delete => {
            let query_result =
                match conn.collection::<HashMap<String, String>>(req.table.as_str()).delete_one(document,None){
                    Ok(_) => String::from("Success"),
                    Err(_) =>String::from("Error")
                };
            query_result.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },
    }
    //	Send back an answer
    let mut res_string_result_serialized = String::new();
    let mut first = true;

    for el in result_serialized {
        if first {
            res_string_result_serialized= format!("{}", el);
            first = false;
        }
        else {
            res_string_result_serialized = format!("{}, {}", res_string_result_serialized, el);
        }
    }
    debug!("Invoker | sent the result {}", res_string_result_serialized);
    child_in.write_all(res_string_result_serialized.as_str().as_bytes());
}

fn server (mut conn: Database, sub_command: Subscription){
    let mut n_reqs = 0;

    loop {
        // Stats on time to serve a request
        let start_time = SystemTime::now();
        // Consuming message
        // TODO waiting for a "close" message to break the loop ?
        let command =  String::from_utf8_lossy(&sub_command.next().unwrap().data).to_string();
        debug!("Invoker | new req received command: {}",command);

        n_reqs = n_reqs +1;
        work(&mut conn, command);
        let duration = SystemTime::now().duration_since(start_time).unwrap();
        info!("Invoker | served the request number {}, in {} ms", n_reqs, duration.as_millis());
    }
}

fn main() {
    env_logger::init();
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("27017".to_string());
    let db = env::var("DB-NAME").unwrap_or("testDB".to_string());
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    let group = env::var("GROUP").unwrap_or("default".to_string());
    let url_db = format!("mongodb://{}:{}", address, port);

    debug!("Invoker | starts");
    debug!("Invoker | URL = {}", url_db);

    // Connection to DB
    let client = Client::with_uri_str(url_db).unwrap();
    let conn = client.database(db.as_str());
    debug!("Invoker | Connected to DB: {:?}", conn);

    // Connection to MOM
    let nc = nats::connect(nats_server.as_str()).unwrap();
    debug!("Invoker | Connected to NATS {:?} ", nc);
    let sub_command = nc.queue_subscribe(&trigger_command, &group).unwrap();
    debug!("Invoker | Sub to command topic {:?}", sub_command);

    server(conn, sub_command);
}
