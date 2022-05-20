var mysql = require('mysql');

var id = process.argv[3];

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
  var sql = " Select * from Customers where firstname ='Mario' AND lastname = 'Rossi' AND id = '" + id +"'";
  
  var before = new Date().getTime();
  con.query(sql, function (err, result) {
    var after = new Date().getTime();
    //latenza 
    var latecy = after - before;
    //print info level
    console.info(latecy);

    if (err) throw err;
    console.log(result);
    process.exit(0);
  });
});
