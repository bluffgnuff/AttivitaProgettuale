var mysql = require('mysql');

var con = mysql.createConnection({
  host: "192.168.10.206",
  port: "3306",
  user: "root",
  password: "root",
  database: "testDB",
});

var id = process.argv[4];

con.connect(function(err) {
  if (err) throw err;
  console.log("Connected!");
  var sql = "INSERT INTO CustomersNoConn (firstname, lastname, id) VALUES ('" + id +"', 'Mario', 'Rossi')";
  
  var before = new Date().getTime();
  con.query(sql, function (err, result) {
    var after = new Date().getTime();
    //latenza 
    var latecy = (after - before)*1000;
    //print info level
    console.info("[DB_LATENCY] "+latecy);

    if (err) throw err;
    console.log("1 record inserted");
    process.exit(0);
  });
});
