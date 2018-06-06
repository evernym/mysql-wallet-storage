use serde_json;
use mysql::Value;

use aurora_storage::SearchOptions;
use errors::error_code::ErrorCode;

#[derive(Debug)]
pub enum Operator {
    And(Vec<Operator>),
    Or(Vec<Operator>),
    Not(Box<Operator>),
    Eq(String, String),
    Neq(String, String),
    Gt(String, String),
    Gte(String, String),
    Lt(String, String),
    Lte(String, String),
    Like(String, String),
    In(String, Vec<String>),
}

impl Operator {
    fn optimise(self) -> Operator {
        match self {
            Operator::Not(boxed_operator) => if let Operator::Not(nested_operator) = *boxed_operator {
                *nested_operator
            } else {
                Operator::Not(boxed_operator)
            },
            Operator::And(mut suboperators) => if suboperators.len() == 1 {
                suboperators.remove(0)
            } else {
                Operator::And(suboperators)
            },
            Operator::Or(mut suboperators) => if suboperators.len() == 1 {
                suboperators.remove(0)
            } else {
                Operator::Or(suboperators)
            },
            Operator::In(key, mut targets) => if targets.len() == 1 {
                Operator::Eq(key, targets.remove(0))
            } else {
                Operator::In(key, targets)
            },
            _ => self
        }
    }
}

pub fn parse_from_json(json: &str) -> Result<Operator, ErrorCode> {
    let parsed_json = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(err) => {
            trace!("Search Query Translation Error: Could not parse JSON WQL Query because: {}", err);
            return Err(ErrorCode::InvalidStructure)
        }
    };

    if let serde_json::Value::Object(map) = parsed_json {
        parse(map)
    } else {
        Err(ErrorCode::InvalidStructure)
    }
}

fn parse(map: serde_json::Map<String, serde_json::Value>) -> Result<Operator, ErrorCode> {
    let mut operators: Vec<Operator> = Vec::new();

    for (key, value) in map.into_iter() {
        let suboperator = parse_operator(key, value)?;
        operators.push(suboperator);
    }

    let top_operator = Operator::And(operators);
    Ok(top_operator.optimise())
}

fn parse_operator(key: String, value: serde_json::Value) -> Result<Operator, ErrorCode> {
    match (&*key, value) {
        ("$or", serde_json::Value::Array(values)) => {
            let mut operators: Vec<Operator> = Vec::new();

            for value in values.into_iter() {
                if let serde_json::Value::Object(map) = value {
                    let suboperator = parse(map)?;
                    operators.push(suboperator);
                } else {
                    warn!("Search Query Translation Error: `$or` operator must be used with an array of JSON objects");
                    return Err(ErrorCode::InvalidStructure);
                }
            }

            Ok(Operator::Or(operators))
        },
        ("$not", serde_json::Value::Object(map)) => {
            let operator = parse(map)?;
            Ok(Operator::Not(Box::new(operator)))
        },
        (_, serde_json::Value::String(value)) => Ok(Operator::Eq(key, value)),
        (_, serde_json::Value::Object(map)) => {
            if map.len() == 1 {
                let (operator_name, value) = map.into_iter().next().unwrap();
                parse_single_operator(operator_name, key, value)
            } else {
                warn!("Search Query Translation Error: `{}` must be used with a JSON object of length 1", key);
                Err(ErrorCode::InvalidStructure)
            }
        },
        (_, _) => {
            warn!("Search Query Translation Error: Unsupported value type for key: `{}`", key);
            Err(ErrorCode::InvalidStructure)
        }
    }
}

fn parse_single_operator(operator_name: String, key: String, value: serde_json::Value) -> Result<Operator, ErrorCode> {
    match (&*operator_name, value) {
        ("$neq", serde_json::Value::String(s)) => Ok(Operator::Neq(key, s)),
        ("$gt", serde_json::Value::String(s)) => Ok(Operator::Gt(key, s)),
        ("$gte", serde_json::Value::String(s)) => Ok(Operator::Gte(key, s)),
        ("$lt", serde_json::Value::String(s)) => Ok(Operator::Lt(key, s)),
        ("$lte", serde_json::Value::String(s)) => Ok(Operator::Lte(key, s)),
        ("$like", serde_json::Value::String(s)) => Ok(Operator::Like(key, s)),
        ("$in", serde_json::Value::Array(values)) => {
            let mut target_values: Vec<String> = Vec::new();

            for v in values.into_iter() {
                if let serde_json::Value::String(s) = v {
                    target_values.push(String::from(s));
                } else {
                    warn!("Search Query Translation Error: `$in` operator must be used with an array of strings");
                    return Err(ErrorCode::InvalidStructure);
                }
            }

            Ok(Operator::In(key, target_values))
        },
        (_, _) => {
            warn!("Search Query Translation Error: Bad operator: {}", operator_name);
            Err(ErrorCode::InvalidStructure)
        }
    }
}

fn operator_to_sql(op: &Operator, arguments: &mut Vec<Value>) -> Result<String, ErrorCode> {
    match *op {
        Operator::Eq(ref tag_name, ref target_value) => Ok(eq_to_sql(tag_name, target_value, arguments)),
        Operator::Neq(ref tag_name, ref target_value) => Ok(neq_to_sql(tag_name, target_value, arguments)),
        Operator::Gt(ref tag_name, ref target_value) => gt_to_sql(tag_name, target_value, arguments),
        Operator::Gte(ref tag_name, ref target_value) => gte_to_sql(tag_name, target_value, arguments),
        Operator::Lt(ref tag_name, ref target_value) => lt_to_sql(tag_name, target_value, arguments),
        Operator::Lte(ref tag_name, ref target_value) => lte_to_sql(tag_name, target_value, arguments),
        Operator::Like(ref tag_name, ref target_value) => like_to_sql(tag_name, target_value, arguments),
        Operator::In(ref tag_name, ref target_values) => Ok(in_to_sql(tag_name, target_values, arguments)),
        Operator::And(ref suboperators) => and_to_sql(suboperators, arguments),
        Operator::Or(ref suboperators) => or_to_sql(suboperators, arguments),
        Operator::Not(ref suboperator) => not_to_sql(suboperator, arguments),
    }
}

fn eq_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> String {

    let tag_path = format!(r#"'$."{}"'"#, tag_name);

    arguments.push(tag_value.into());
    format!("(JSON_UNQUOTE(JSON_EXTRACT(tags, {})) = ?)", tag_path)
}

fn neq_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> String {

    let tag_path = format!(r#"'$."{}"'"#, tag_name);

    arguments.push(tag_value.into());
    format!("(JSON_UNQUOTE(JSON_EXTRACT(tags, {})) != ?)", tag_path)
}

fn gt_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Result<String, ErrorCode> {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            let tag_path = format!(r#"'$."{}"'"#, tag_name);

            arguments.push(tag_value.into());
            Ok(format!("(JSON_UNQUOTE(JSON_EXTRACT(tags, {})) > ?)", tag_path))
        },
        _ => {
            warn!("Search Query Translation Error: Trying to use `gt` operator with a encrypted tag");
            Err(ErrorCode::InvalidStructure)
        }
    }
}

fn gte_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Result<String, ErrorCode> {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            let tag_path = format!(r#"'$."{}"'"#, tag_name);

            arguments.push(tag_value.into());
            Ok(format!("(JSON_UNQUOTE(JSON_EXTRACT(tags, {})) >= ?)", tag_path))
        },
        _ => {
            warn!("Search Query Translation Error: Trying to use `gte` operator with a encrypted tag");
            Err(ErrorCode::InvalidStructure)
        }
    }
}

fn lt_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Result<String, ErrorCode> {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            let tag_path = format!(r#"'$."{}"'"#, tag_name);

            arguments.push(tag_value.into());
            Ok(format!("JSON_UNQUOTE(JSON_EXTRACT(tags, {})) < ?)", tag_path))
        },
        _ => {
            warn!("Search Query Translation Error: Trying to use `lt` operator with a encrypted tag");
            Err(ErrorCode::InvalidStructure)
        }
    }
}

fn lte_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Result<String, ErrorCode> {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            let tag_path = format!(r#"'$."{}"'"#, tag_name);

            arguments.push(tag_value.into());
            Ok(format!("(JSON_UNQUOTE(JSON_EXTRACT(tags, {})) <= ?)", tag_path))
        },
        _ => {
            warn!("Search Query Translation Error: Trying to use `lte` operator with a encrypted tag");
            Err(ErrorCode::InvalidStructure)
        }
    }
}

fn like_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Result<String, ErrorCode> {
   match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            let tag_path = format!(r#"'$."{}"'"#, tag_name);

            arguments.push(tag_value.into());
            Ok(format!("(JSON_UNQUOTE(JSON_EXTRACT(tags, {})) LIKE ?)", tag_path))
        },
        _ => {
            warn!("Search Query Translation Error: Trying to use `like` operator with a encrypted tag");
            Err(ErrorCode::InvalidStructure)
        }
    }
}

fn in_to_sql(tag_name: &String, tag_values: &Vec<String>, arguments: &mut Vec<Value>) -> String {

    let tag_path = format!(r#"'$."{}"'"#, tag_name);
    let mut in_string = format!("JSON_UNQUOTE(JSON_EXTRACT(tags, {})) IN (", tag_path);

    for (index, tag_value) in tag_values.iter().enumerate() {
        in_string.push_str("?");
        if index < tag_values.len() - 1 {
            in_string.push(',');
        }
        else {
            in_string.push(')');
        }

        arguments.push(tag_value.into());
    }

    in_string
}

fn and_to_sql(suboperators: &[Operator], arguments: &mut Vec<Value>) -> Result<String, ErrorCode> {
    join_operators(suboperators, " AND ", arguments)
}

fn or_to_sql(suboperators: &[Operator], arguments: &mut Vec<Value>) -> Result<String, ErrorCode> {
    join_operators(suboperators, " OR ", arguments)
}

fn not_to_sql(suboperator: &Operator, arguments: &mut Vec<Value>) -> Result<String, ErrorCode> {
    let suboperator_string = operator_to_sql(suboperator, arguments);

    match suboperator_string {
        Ok(suboperator_string) => Ok("NOT (".to_string() + &suboperator_string + ")"),
        Err(err) => return Err(err)
    }
}

fn join_operators(operators: &[Operator], join_str: &str, arguments: &mut Vec<Value>) -> Result<String, ErrorCode> {
    let mut s = String::new();

    if !operators.is_empty() {
        s.push('(');
        for (index, operator) in operators.iter().enumerate() {
            let operator_string = operator_to_sql(operator, arguments);

            match operator_string {
                Ok(operator_string) => s.push_str(&operator_string),
                Err(err) => return Err(err)
            }

            if index < operators.len() - 1 {
                s.push_str(join_str);
            }
        }
        s.push(')');
    }

    Ok(s)
}

pub fn wql_to_sql(wallet_id: u64, type_: &str, wql: &Operator, options: &SearchOptions) -> Result<(String, Vec<Value>), ErrorCode> {

    trace!("Translating WQL to SQL Fetch Query -> type: {}, wql: {:?}, options: {:?}", type_, wql, options);

    let mut arguments: Vec<Value> = Vec::new();
    let query_condition = match operator_to_sql(wql, &mut arguments) {
        Ok(query_condition) => query_condition,
        Err(err) => return Err(err)
    };

    let query_string = format!(
        "SELECT {}, name, {}, {} FROM items WHERE {} type = ? AND wallet_id = ?",
        if options.retrieve_type { "type" } else {"NULL"},
        if options.retrieve_value { "value" } else {"NULL"},
        if options.retrieve_tags { "tags" } else {"NULL"},
        if !query_condition.is_empty() {query_condition + " AND"} else {"".to_string()}
    );

    arguments.push(type_.into());
    arguments.push(wallet_id.into());

    trace!("Success Translating WQL: {:?} to SQL Fetch Query -> query: {}, args: {:?}", wql, query_string, arguments);

    Ok((query_string, arguments))
}

pub fn wql_to_sql_count(wallet_id: u64, type_: &str, wql: &Operator) -> Result<(String, Vec<Value>), ErrorCode> {

    trace!("Translating WQL to SQL Count Query -> type: {}, wql: {:?}", type_, wql);

    let mut arguments: Vec<Value> = Vec::new();
    let query_condition = match operator_to_sql(wql, &mut arguments) {
        Ok(query_condition) => query_condition,
        Err(err) => return Err(err)
    };

    let query_string = format!(
        "SELECT count(*) FROM items i WHERE {} i.type = ? AND i.wallet_id = ?",
        if !query_condition.is_empty() {query_condition + " AND"} else {"".to_string()}
    );

    arguments.push(type_.into());
    arguments.push(wallet_id.into());

    trace!("Success Translating WQL: {:?} to SQL Count Query -> query: {}, args: {:?}", wql, query_string, arguments);

    Ok((query_string, arguments))
}
