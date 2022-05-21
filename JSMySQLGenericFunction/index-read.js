var mysql = require('mysql');

var id = process.argv[4];

var con = mysql.createConnection({
  host: "192.168.10.206",
  port: "3306",
  user: "root",
  password: "root",
  database: "testDB",
  //insecure_auth: true,
  //ssl: {rejectUnauthorized: false}
});

con.connect(function(err) {
  if (err) throw err;
  console.log("Connected!");
  var sql = " Select * from CustomersNoConn where firstname ='Mario' AND lastname = 'Rossi' AND id = '" + id +"'";
  
  var before = new Date().getTime();
  con.query(sql, function (err, result) {
    var after = new Date().getTime();
    //latenza 
    var latecy = (after - before)*1000;

    //print info level
    console.info("[DB_LATENCY] "+latecy);

    if (err) throw err;
    console.log(result);
    process.exit(0);
  });
});
