use mysql::prelude::*;
use mysql::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use serde;
#[macro_use]
use serde::{Deserialize, Serialize};
use rmps::{Deserializer, Serializer};

//Vincoli:
// Devo strutturare un invoker per un MoM che prenda anche il dato.
// Il processo crea un figlio con cui comunica con stdin.
// Il figlio (funzione serverless) cheide al padre un operazione CRUD (chiede un opzione CRUD)
// Il padre tiene la connessione al DB.
// utilizzare RMP per comunicare tra i processi
//Note:
//let mut req = String::from("INSERT INTO Customers(ID, FirstName, LastName) VALUES ('1', 'Mario', 'Rossi')");
//let req = String::from("SELECT Lastname FROM Customers;");
//let req = String::from(" UPDATE Customers SET FirstName = 'Luca' WHERE FirstName = 'Mario';");
//let req = String::from("DELETE from Customers where LastName='Rossi';");
//Problema:
//Posso tentare usando solo la stdin (della stdout non mi interessa)

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Customer {
    firstname: String,
    lastname: String,
}
#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum Op {
    Create,
    Read,
    Update,
    Delete,
}
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Request {
    op: Op,
    customer: Option<Customer>,
    optional: Option<Customer>,
}

fn server(url: &str, rx: Receiver<String>) {
    let opts = Opts::from_url(url);
    let pool = Pool::new(opts.unwrap()).unwrap();
    let mut conn = pool.get_conn().unwrap();

    loop {
        let mut req = rx.recv().unwrap();

        println!("[P] Request received: {}", req);
        let res: Vec<String> = conn.query(req).unwrap();
        print!("[P]Query result:");
        for el in res {
            print!("{} ", el);
        }
        print!("\n");
    }
}

fn main() {
    let url = "mysql://root:root@127.0.0.1:3306/testDB";

    let opts = Opts::from_url(url);
    let pool = Pool::new(opts.unwrap()).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    //stdout((stdin));

    let son = thread::spawn(move || {
        // Create
        let mut customer = Customer {
            firstname: String::from("Mario"),
            lastname: String::from("Rossi"),
        };

        let mut req = Request {
            op: Op::Create,
            customer: Option::from(customer),
            optional: None,
        };
        let req_pack = None;
        //let mut req = String::from("INSERT INTO Customers(ID, FirstName, LastName) VALUES ('1', 'Mario', 'Rossi')");
        //println!("[C] asks for a {}", req);
        //tx.send(req_pack);
        thread::sleep(Duration::from_millis(100));

        // Read
        customer = Customer {
            firstname: String::from("Mario"),
            lastname: String::from("Rossi"),
        };

        req = Request {
            op: Op::Read,
            customer: Option::from(customer),
            optional: None,
        };

        //let req = String::from("SELECT Lastname FROM Customers;");
        //println!("[C] asks for a {}", req);
        //tx.send(req_pack);
        thread::sleep(Duration::from_millis(100));

        // Update
        customer = Customer {
            firstname: String::from("Mario"),
            lastname: String::from("Rossi"),
        };

        let customer2 = Customer {
            firstname: String::from("Luca"),
            lastname: String::from("Rossi"),
        };

        req = Request {
            op: Op::Update,
            customer: Option::from(customer),
            optional: Option::from(customer2),
        };
        //let req = String::from(" UPDATE Customers SET FirstName = 'Luca' WHERE FirstName = 'Mario';");
        //println!("[C] asks for an {}", req);
        //tx.send(req_pack);
        thread::sleep(Duration::from_millis(100));

        // Delete
        customer = Customer {
            firstname: String::from("Luca"),
            lastname: String::from("Rossi"),
        };

        req = Request {
            op: Op::Delete,
            customer: Option::from(customer),
            optional: None,
        };
        //let req = String::from("DELETE from Customers where LastName='Rossi';");
        //println!("[C] asks for a {}", req);
        //tx.send(req_pack);
    });

    server(url, rx);
}

// let mut buf = Vec::new();
// let val = Human {
// age: 42,
// name: "John".into(),
// };
//
// val.serialize(&mut Serializer::new(&mut buf)).unwrap();