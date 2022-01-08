//MySQL
use mysql::*;
use mysql::prelude::*;

fn main() {
    let url = "mysql://root:root@127.0.0.1:3306/testDB";

    let opts = Opts::from_url(url);
    let pool =Pool::new(opts.unwrap()).unwrap();

    let mut conn = pool.get_conn();

    let row: Vec<String>  = conn.unwrap().query("SELECT Lastname FROM Customers;").unwrap();

    println!("{}",row.get(0).unwrap());
}