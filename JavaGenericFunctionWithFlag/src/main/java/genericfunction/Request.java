package genericfunction;

import java.util.HashMap;
import java.util.Optional;

public class Request {
    private Op op;
    private String table;
    private HashMap<String, String> param;
    private Optional<HashMap<String, String>> param_to_up;

    public Request(){};
    public Request(Op op, String table, HashMap<String, String> param, Optional<HashMap<String, String>> param_to_up) {
        this.op = op;
        this.table = table;
        this.param = param;
        this.param_to_up = param_to_up;
    }

    public Op getOp() {
        return op;
    }

    public void setOp(Op op) {
        this.op = op;
    }

    public String getTable() {
        return table;
    }

    public void setTable(String table) {
        this.table = table;
    }

    public HashMap<String, String> getParam() {
        return param;
    }

    public void setParam(HashMap<String, String> param) {
        this.param = param;
    }

    public Optional<HashMap<String, String>> getParam_to_up() {
        return param_to_up;
    }

    public void setParam_to_up(Optional<HashMap<String, String>> param_to_up) {
        this.param_to_up = param_to_up;
    }
}
