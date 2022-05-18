package mysql.function;

import org.apache.commons.cli.*;

import java.sql.*;
import java.util.HashMap;
import java.util.Map;
import java.util.Scanner;

public class JavaMySQLGenericFunction {

	public static void main(String[] args) {
		Map<String, String> env = System.getenv();
		String username = env.getOrDefault("NAME", "root");
		String password = env.getOrDefault("PASSWORD", "root");
		String address = env.getOrDefault("ADDRESS", "127.0.0.1");
		String port = env.getOrDefault("PORT", "3306");
		String db_name = env.getOrDefault("DB-NAME", "testDB");
		String url_db = "mysql://" + address + ":" + port + "/" + db_name;

		HashMap<String, String> customer = new HashMap<String, String>();
		HashMap<String, String> customer_new = new HashMap<String, String>();
		String table = "";
		String operation = "";

		CommandLine commandLine;
//		Controllo argomenti
		Options options = new Options();
		Option OptionTable = Option.builder("t")
				.required(false)
				.desc("The table in which work")
				.longOpt("table")
				.hasArg()
				.build();
		options.addOption(OptionTable);

		Option OptionOperation = Option.builder("o")
				.required(false)
				.desc("The operation to execute")
				.longOpt("operation")
				.hasArg()
				.build();
		options.addOption(OptionOperation);

		// added an id column (not auto increment) in the DBs so the client can add it manually
		Option OptionId = Option.builder("i")
				.required(false)
				.desc("The new entry ID")
				.longOpt("id")
				.hasArg()
				.build();
		options.addOption(OptionId);

		Option OptionFirstname = Option.builder("f")
				.required(false)
				.desc("The new entry firstname")
				.longOpt("firstname")
				.hasArg()
				.build();
		options.addOption(OptionFirstname);

		Option OptionLastname = Option.builder("l")
				.required(false)
				.desc("The new entry lastname")
				.longOpt("lastname")
				.hasArg()
				.build();
		options.addOption(OptionLastname);

		Option OptionLastname_opt = Option.builder("lo")
				.required(false)
				.desc("The the new lastname for the old entry")
				.longOpt("lastname_opt")
				.hasArg()
				.build();
		options.addOption(OptionLastname_opt);

		Option OptionFirstname_opt = Option.builder("fo")
				.required(false)
				.desc("The the new firstname for the old entry")
				.longOpt("firstname_opt")
				.hasArg()
				.build();
		options.addOption(OptionFirstname_opt);

		CommandLineParser commandLineParser = new DefaultParser();
		try {
			commandLine = commandLineParser.parse(options, args);
		} catch (ParseException e) {
			throw new RuntimeException(e);
		}

		if (commandLine.hasOption("table")) {
			table = commandLine.getOptionValue("table");
		}

		if (commandLine.hasOption("operation")) {
			operation = commandLine.getOptionValue("operation");
		}

		if (commandLine.hasOption("id")) {
			customer.put("id", commandLine.getOptionValue("id"));
		}
//        if (commandLine.hasOption("_rev")){
//        }

		if (commandLine.hasOption("firstname")) {
			customer.put("firstname", commandLine.getOptionValue("firstname"));
		}

		if (commandLine.hasOption("lastname")) {
			customer.put("lastname", commandLine.getOptionValue("lastname"));
		}

		if (commandLine.hasOption("lastname_opt")) {
			customer_new.put("lastname_opt", commandLine.getOptionValue("lastname_opt"));
		}

		if (commandLine.hasOption("firstname_opt")) {
			customer_new.put("firstname_opt", commandLine.getOptionValue("firstname_opt"));
		}

//       Connection to MySQL DB
		try {
			Connection conn = DriverManager.getConnection("jdbc:" + url_db, username, password);
			StringBuilder queryBuilder = new StringBuilder();
			String query;

//          TODO Extend to UPDATE and DELETE
			if (operation.equals("Create")) {
				Boolean first = true;
				StringBuilder colsBuilder = new StringBuilder();
				StringBuilder valsBuilder = new StringBuilder();
				queryBuilder.append("Insert into ");
				queryBuilder.append(table);
				colsBuilder.append(" ( ");
				valsBuilder.append("Values ( ");

				for (String col : customer.keySet()) {
					if (!first) {
						colsBuilder.append(", ");
						valsBuilder.append(", ");
					}
					valsBuilder.append("'");
					String column_val = customer.get(col);
					colsBuilder.append(col);
					valsBuilder.append(column_val);

					if (first) {
						first = false;
					}
					valsBuilder.append("'");
				}
				colsBuilder.append(" ) ");
				valsBuilder.append(" )");
				queryBuilder.append(colsBuilder);
				queryBuilder.append(valsBuilder);
				query = queryBuilder.toString();
			} else {
//            if (operation == "Read")
				Boolean first = true;
				queryBuilder.append("Select * From ");
				queryBuilder.append(table);
				queryBuilder.append(" Where ");

				for (String col : customer.keySet()) {
					if (!first) {
						queryBuilder.append(" AND ");
					}

					String val = customer.get(col);
					queryBuilder.append(col);
					queryBuilder.append("=");
					queryBuilder.append("'");
					queryBuilder.append(val);
					queryBuilder.append("'");
					if (first) {
						first = false;
					}
				}
				query = queryBuilder.toString();
			}
//			Answer to Invoker
			if(operation.equals("Create")){
				Statement stm = conn.prepareStatement(query);
				stm.execute(query);
				String result = "ESEGUITO";
				System.out.println(result);
			}else{
				Statement stm = conn.createStatement();
				ResultSet result = stm.executeQuery(query);
				while(result.next()){
					for(String col : customer.keySet()) {
						System.out.print(result.getString(col) +" ");
					}
				}
			}
		} catch (SQLException e) {
			throw new RuntimeException(e);
		}

	}
}