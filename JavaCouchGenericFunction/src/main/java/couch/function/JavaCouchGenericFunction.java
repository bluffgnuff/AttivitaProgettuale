package couch.function;

import org.apache.commons.cli.*;
import org.json.JSONObject;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.MalformedURLException;
import java.net.ProtocolException;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.HashMap;
import java.util.Map;
import java.util.Scanner;

//--table Customers --operation Read --id idProva2 --firstname NomeProva --lastname CognomeProva
public class JavaCouchGenericFunction {

	public static void main(String[] args) {
		Map<String, String> env = System.getenv();
		String username = env.getOrDefault("NAME", "root");
		String password = env.getOrDefault("PASSWORD", "root");
		String address = env.getOrDefault("ADDRESS", "127.0.0.1");
		String port = env.getOrDefault("PORT", "5984");
		String db_name = env.getOrDefault("DB-NAME", "testdb");
		String url_db = "http://" + address + ":" + port + "/" + db_name;

		Scanner stdin = new Scanner(System.in);
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
        if (commandLine.hasOption("_rev")){
			customer.put("_rev", commandLine.getOptionValue("_rev"));
        }

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

//       Connection HTTP to Couch DB
		try {

//          TODO Extend to UPDATE, DELETE
			if (operation.equals("Create")) {
				URL url = new URL(url_db);
				HttpURLConnection con = (HttpURLConnection) url.openConnection();
				String encoding = Base64.getEncoder().encodeToString((username + ":" + password).getBytes(StandardCharsets.UTF_8));
				con.setRequestProperty("Authorization", "Basic " + encoding);
				con.setRequestMethod("POST");
				con.setDoOutput(true);
				con.setRequestProperty("Content-Type", "application/json; utf-8");

				JSONObject selector = new JSONObject();
				for (var entry : customer.entrySet()) {
					selector.put(entry.getKey(), entry.getValue());
				}
//				Send Request
				try(OutputStream os = con.getOutputStream()) {
					byte[] input = selector.toString().getBytes("utf-8");
					os.write(input, 0, input.length);
				}
//				Receive Response
				try(BufferedReader br = new BufferedReader(
						new InputStreamReader(con.getInputStream(), "utf-8"))) {
					StringBuilder response = new StringBuilder();
					String responseLine = null;
					while ((responseLine = br.readLine()) != null) {
						response.append(responseLine.trim());
					}
					System.out.println(response.toString());
				}
			} else {
				URL url = new URL(url_db+"/_find");
				HttpURLConnection con = (HttpURLConnection) url.openConnection();
				String encoding = Base64.getEncoder().encodeToString((username + ":" + password).getBytes(StandardCharsets.UTF_8));
				con.setRequestProperty("Authorization", "Basic " + encoding);
				con.setRequestMethod("POST");
				con.setDoOutput(true);
				con.setRequestProperty("Content-Type", "application/json; utf-8");

				JSONObject selector = new JSONObject();
				for (var entry : customer.entrySet()) {
					JSONObject eq_line = new JSONObject();
					eq_line.put("$eq", entry.getValue());
					selector.put(entry.getKey(), eq_line);
				}

				JSONObject document = new JSONObject();
				document.put("selector", selector);

//				Send Request
				try(OutputStream os = con.getOutputStream()) {
					byte[] input = document.toString().getBytes("utf-8");
					os.write(input, 0, input.length);
				}

//				Receive Response
				try(BufferedReader br = new BufferedReader(
						new InputStreamReader(con.getInputStream(), "utf-8"))) {
					StringBuilder response = new StringBuilder();
					String responseLine = null;
					while ((responseLine = br.readLine()) != null) {
						response.append(responseLine.trim());
					}
					System.out.println(response.toString());
				}
			}
		} catch (NumberFormatException e) {
			throw new RuntimeException(e);
		} catch (ProtocolException e) {
			throw new RuntimeException(e);
		} catch (MalformedURLException e) {
			throw new RuntimeException(e);
		} catch (IOException e) {
			throw new RuntimeException(e);
		}
	}
}