var mysql = require('mysql');

var con = mysql.createConnection({
  host: "192.168.10.206",
  port: "3306",
  user: "root",
  password: "root",
  database: "testDB",
});

var id = process.argv[3];

con.connect(function(err) {
  if (err) throw err;
  console.log("Connected!");
  var sql = "INSERT INTO Customers (firstname, lastname, id) VALUES ('" + id +"', 'Mario', 'Rossi')";
  
  var before = new Date().getTime();
  con.query(sql, function (err, result) {
    var after = new Date().getTime();
    //latenza 
    var latecy = after - before;
    //print info level
    console.info(latecy);

    if (err) throw err;
    console.log("1 record inserted");
    process.exit(0);
  });
});
