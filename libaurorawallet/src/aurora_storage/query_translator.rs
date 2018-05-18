use std::string;
use serde_json;
use mysql::Value;

use aurora_storage::FetchOptions;

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
    Regex(String, String),
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

fn join_operator_strings(operators: &Vec<Operator>) -> String {
    operators.iter()
             .map(|o: &Operator| -> String { o.to_string() })
             .collect::<Vec<String>>()
             .join(",")
}

impl string::ToString for Operator {
    fn to_string(&self) -> String {
        match *self {
            Operator::Eq(ref tag_name, ref tag_value) => format!("\"{}\": \"{}\"", tag_name.to_string(), tag_value.to_string()),
            Operator::Neq(ref tag_name, ref tag_value) => format!("\"{}\": {{\"$neq\": \"{}\"}}", tag_name.to_string(), tag_value.to_string()),
            Operator::Gt(ref tag_name, ref tag_value) => format!("\"{}\": {{\"$gt\": \"{}\"}}", tag_name.to_string(), tag_value.to_string()),
            Operator::Gte(ref tag_name, ref tag_value) => format!("\"{}\": {{\"$gte\": \"{}\"}}", tag_name.to_string(), tag_value.to_string()),
            Operator::Lt(ref tag_name, ref tag_value) => format!("\"{}\": {{\"$lt\": \"{}\"}}", tag_name.to_string(), tag_value.to_string()),
            Operator::Lte(ref tag_name, ref tag_value) => format!("\"{}\": {{\"$lte\": \"{}\"}}", tag_name.to_string(), tag_value.to_string()),
            Operator::Like(ref tag_name, ref tag_value) => format!("\"{}\": {{\"$like\": \"{}\"}}", tag_name.to_string(), tag_value.to_string()),
            Operator::Regex(ref tag_name, ref tag_value) => format!("\"{}\": {{\"$regex\": \"{}\"}}", tag_name.to_string(), tag_value.to_string()),
            Operator::Not(ref stmt) => format!("\"$not\": {{{}}}", stmt.to_string()),
            Operator::And(ref operators) => format!("{{{}}}", join_operator_strings(operators)),
            Operator::Or(ref operators) => format!("\"$or\": [{}])", join_operator_strings(operators)),
            Operator::In(ref tag_name, ref tag_values) => {
                let strings = tag_values.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(", ");
                format!("\"{}\": {{\"$in\": [{}]}}", tag_name.to_string(), strings)
            },
        }
    }
}

pub fn parse_from_json(json: &str) -> Option<Operator> {
    let parsed_json = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(_) => return None
    };

    if let serde_json::Value::Object(map) = parsed_json {
        parse(map)
    } else {
        None
    }
}

fn parse(map: serde_json::Map<String, serde_json::Value>) -> Option<Operator> {
    let mut operators: Vec<Operator> = Vec::new();

    for (key, value) in map.into_iter() {
        let suboperator = parse_operator(key, value)?;
        operators.push(suboperator);
    }

    let top_operator = Operator::And(operators);
    Some(top_operator.optimise())
}

fn parse_operator(key: String, value: serde_json::Value) -> Option<Operator> {
    match (&*key, value) {
        ("$or", serde_json::Value::Array(values)) => {
            let mut operators: Vec<Operator> = Vec::new();

            for value in values.into_iter() {
                if let serde_json::Value::Object(map) = value {
                    let suboperator = parse(map)?;
                    operators.push(suboperator);
                } else {
                    return None;
                }
            }

            Some(Operator::Or(operators))
        },
        ("$not", serde_json::Value::Object(map)) => {
            let operator = parse(map)?;
            Some(Operator::Not(Box::new(operator)))
        },
        (_, serde_json::Value::String(value)) => Some(Operator::Eq(key, value)),
        (_, serde_json::Value::Object(map)) => {
            if map.len() == 1 {
                let (operator_name, value) = map.into_iter().next().unwrap();
                parse_single_operator(operator_name, key, value)
            } else {
                None
            }
        },
        (_, _) => None
    }
}

fn parse_single_operator(operator_name: String, key: String, value: serde_json::Value) -> Option<Operator> {
    match (&*operator_name, value) {
        ("$neq", serde_json::Value::String(s)) => Some(Operator::Neq(key, s)),
        ("$gt", serde_json::Value::String(s)) => Some(Operator::Gt(key, s)),
        ("$gte", serde_json::Value::String(s)) => Some(Operator::Gte(key, s)),
        ("$lt", serde_json::Value::String(s)) => Some(Operator::Lt(key, s)),
        ("$lte", serde_json::Value::String(s)) => Some(Operator::Lte(key, s)),
        ("$like", serde_json::Value::String(s)) => Some(Operator::Like(key, s)),
        ("$regex", serde_json::Value::String(s)) => Some(Operator::Regex(key, s)),
        ("$in", serde_json::Value::Array(values)) => {
            let mut target_values: Vec<String> = Vec::new();

            for v in values.into_iter() {
                if let serde_json::Value::String(s) = v {
                    target_values.push(String::from(s));
                } else {
                    return None;
                }
            }

            Some(Operator::In(key, target_values))
        },
        (_, _) => None
    }
}

fn operator_to_sql(op: &Operator, arguments: &mut Vec<Value>) -> Option<String> {
    match *op {
        Operator::Eq(ref tag_name, ref target_value) => Some(eq_to_sql(tag_name, target_value, arguments)),
        Operator::Neq(ref tag_name, ref target_value) => Some(neq_to_sql(tag_name, target_value, arguments)),
        Operator::Gt(ref tag_name, ref target_value) => gt_to_sql(tag_name, target_value, arguments),
        Operator::Gte(ref tag_name, ref target_value) => gte_to_sql(tag_name, target_value, arguments),
        Operator::Lt(ref tag_name, ref target_value) => lt_to_sql(tag_name, target_value, arguments),
        Operator::Lte(ref tag_name, ref target_value) => lte_to_sql(tag_name, target_value, arguments),
        Operator::Like(ref tag_name, ref target_value) => like_to_sql(tag_name, target_value, arguments),
        Operator::Regex(ref tag_name, ref target_value) => regex_to_sql(tag_name, target_value, arguments),
        Operator::In(ref tag_name, ref target_values) => Some(in_to_sql(tag_name, target_values, arguments)),
        Operator::And(ref suboperators) => and_to_sql(suboperators, arguments),
        Operator::Or(ref suboperators) => or_to_sql(suboperators, arguments),
        Operator::Not(ref suboperator) => not_to_sql(suboperator, arguments),
    }
}

fn eq_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> String {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            arguments.push(tag_name[1..].into());
            arguments.push(tag_value.into());
            "(i.id in (SELECT item_id FROM tags_plaintext WHERE name = ? AND value = ?))".to_string()
        },
        _ => {
            arguments.push(tag_name.into());
            arguments.push(tag_value.into());
            "(i.id in (SELECT item_id FROM tags_encrypted WHERE name = ? AND value = ?))".to_string()
        }
    }
}

fn neq_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> String {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            arguments.push(tag_name[1..].into());
            arguments.push(tag_value.into());
            "(i.id in (SELECT item_id FROM tags_plaintext WHERE name = ? AND value != ?))".to_string()
        },
        _ => {
            arguments.push(tag_name.into());
            arguments.push(tag_value.into());
            "(i.id in (SELECT item_id FROM tags_encrypted WHERE name = ? AND value != ?))".to_string()
        }
    }
}

fn gt_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Option<String> {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            arguments.push(tag_name[1..].into());
            arguments.push(tag_value.into());
            Some("(i.id in (SELECT item_id FROM tags_plaintext WHERE name = ? AND value > ?))".to_string())
        },
        _ => None
    }
}

fn gte_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Option<String> {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            arguments.push(tag_name[1..].into());
            arguments.push(tag_value.into());
            Some("(i.id in (SELECT item_id FROM tags_plaintext WHERE name = ? AND value >= ?))".to_string())
        },
        _ => None
    }
}

fn lt_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Option<String> {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            arguments.push(tag_name[1..].into());
            arguments.push(tag_value.into());
            Some("(i.id in (SELECT item_id FROM tags_plaintext WHERE name = ? AND value < ?))".to_string())
        },
        _ => None
    }
}

fn lte_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Option<String> {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            arguments.push(tag_name[1..].into());
            arguments.push(tag_value.into());
            Some("(i.id in (SELECT item_id FROM tags_plaintext WHERE name = ? AND value <= ?))".to_string())
        },
        _ => None
    }
}

fn like_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Option<String> {
   match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            arguments.push(tag_name[1..].into());
            arguments.push(tag_value.into());
            Some("(i.id in (SELECT item_id FROM tags_plaintext WHERE name = ? AND value LIKE ?))".to_string())
        },
        _ => None
    }
}

fn regex_to_sql(tag_name: &String, tag_value: &String, arguments: &mut Vec<Value>) -> Option<String> {
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            arguments.push(tag_name[1..].into());
            arguments.push(tag_value.into());
            Some("(i.id in (SELECT item_id FROM tags_plaintext WHERE name = ? AND value REGEXP ?))".to_string())
        },
        _ => None
    }
}

fn in_to_sql(tag_name: &String, tag_values: &Vec<String>, arguments: &mut Vec<Value>) -> String {
    let mut in_string = String::new();
    match tag_name.chars().next().unwrap_or('\0') {
        '~' => {
            in_string.push_str("(i.id in (SELECT item_id FROM tags_plaintext WHERE name = ? AND value IN (");
            arguments.push(tag_name[1..].into());

            for (index, tag_value) in tag_values.iter().enumerate() {
                in_string.push_str("?");
                arguments.push(tag_value.into());
                if index < tag_values.len() - 1 {
                    in_string.push(',');
                }

            }

            in_string + ")))"
        },
        _ => {
            in_string.push_str("(i.id in (SELECT item_id FROM tags_encrypted WHERE name = ? AND value IN (");
            arguments.push(tag_name.into());
            let index_before_last = tag_values.len() - 2;

            for (index, tag_value) in tag_values.iter().enumerate() {
                in_string.push_str("?");
                arguments.push(tag_value.into());
                if index <= index_before_last {
                    in_string.push(',');
                }
            }

            in_string + ")))"
        },
    }
}

fn and_to_sql(suboperators: &[Operator], arguments: &mut Vec<Value>) -> Option<String> {
    join_operators(suboperators, " AND ", arguments)
}

fn or_to_sql(suboperators: &[Operator], arguments: &mut Vec<Value>) -> Option<String> {
    join_operators(suboperators, " OR ", arguments)
}

fn not_to_sql(suboperator: &Operator, arguments: &mut Vec<Value>) -> Option<String> {
    let suboperator_string = operator_to_sql(suboperator, arguments);

    match suboperator_string {
        Some(suboperator_string) => Some("NOT (".to_string() + &suboperator_string + ")"),
        None => return None
    }
}

fn join_operators(operators: &[Operator], join_str: &str, arguments: &mut Vec<Value>) -> Option<String> {
    let mut s = String::new();
    s.push('(');
    for (index, operator) in operators.iter().enumerate() {
        let operator_string = operator_to_sql(operator, arguments);

        match operator_string {
            Some(operator_string) => s.push_str(&operator_string),
            None => return None
        }

        if index < operators.len() - 1 {
            s.push_str(join_str);
        }
    }
    s.push(')');
    Some(s)
}

// Translates Wallet Query Language to SQL
// WQL input is provided as a reference to a top level Operator
// Result is a tuple of query string and query arguments
pub fn wql_to_sql(wallet_id: u64, type_: &str, wql: &Operator, options: &FetchOptions) -> Option<(String, Vec<Value>)> {
    let mut arguments: Vec<Value> = Vec::new();
    let query_condition = match operator_to_sql(wql, &mut arguments) {
        Some(query_condition) => query_condition,
        None => return None
    };

    let query_string = format!(
        "SELECT {}, i.name, {}, {} FROM items i WHERE {} AND i.type = ? AND i.wallet_id = ?;",
        if options.fetch_type { "i.type" } else {"NULL"},
        if options.fetch_value { "i.value" } else {"NULL"},
        if options.fetch_tags {
            "CONCAT(\
                '{', \
                CONCAT_WS(\
                    ',', \
                    (select group_concat(concat(json_quote(name), ':', json_quote(value))) from tags_encrypted WHERE item_id = i.id), \
                    (select group_concat(concat(json_quote(concat('~', name)), ':', json_quote(value))) from tags_plaintext WHERE item_id = i.id)\
                ), \
            '}') tags"
        } else {"NULL"},
        query_condition
    );

    arguments.push(type_.into());
    arguments.push(wallet_id.into());
    Some((query_string, arguments))
}
