use serde_json;
use mysql::Value;

use mysql_storage::SearchOptions;
use errors::error_code::ErrorCode;

#[derive(Debug, Hash, Clone)]
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
        ("$and", serde_json::Value::Array(values)) => {
            let mut operators: Vec<Operator> = Vec::new();

            for value in values.into_iter() {
                if let serde_json::Value::Object(map) = value {
                    let suboperator = parse(map)?;
                    operators.push(suboperator);
                } else {
                    warn!("Search Query Translation Error: `$and` operator must be used with an array of JSON objects");
                    return Err(ErrorCode::InvalidStructure);
                }
            }

            Ok(Operator::And(operators))
        },
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

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use self::rand::{thread_rng, Rng};
    use std::hash::Hash;
    use std::collections::HashSet;

    fn random_string(len: usize) -> String {
        thread_rng().gen_ascii_chars().take(len).collect()
    }

    fn vec_to_set<T>(vec: &Vec<T>) -> HashSet<T> where T: Eq + Hash + Clone {
        let mut result: HashSet<T> = HashSet::new();
        for x in vec {
            result.insert((*x).clone());
        }
        result
    }

    impl PartialEq for Operator {
        fn eq(&self, other: &Operator) -> bool {
            match (self, other) {
                (Operator::Eq(name, value), Operator::Eq(other_name, other_value))
                | (Operator::Neq(name, value), Operator::Neq(other_name, other_value))
                | (Operator::Gt(name, value), Operator::Gt(other_name, other_value))
                | (Operator::Gte(name, value), Operator::Gte(other_name, other_value))
                | (Operator::Lt(name, value), Operator::Lt(other_name, other_value))
                | (Operator::Lte(name, value), Operator::Lte(other_name, other_value))
                | (Operator::Like(name, value), Operator::Like(other_name, other_value)) => {
                    name == other_name && value == other_value
                },
                (Operator::In(name, values), Operator::In(other_name, other_values)) => {
                    name == other_name && vec_to_set(values) == vec_to_set(other_values)
                },
                (Operator::Not(operator), Operator::Not(other_operator)) => operator == other_operator,
                (Operator::And(operators), Operator::And(other_operators))
                | (Operator::Or(operators), Operator::Or(other_operators)) => {
                    vec_to_set(operators) == vec_to_set(other_operators)
                },
                (_, _) => false
            }
        }
    }

    impl Eq for Operator {}

    #[test]
    fn test_simple_operator_empty_json_parse() {
        let json = "{}";

        let query = parse_from_json(json).unwrap();

        let expected = Operator::And(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_explicit_empty_and_parse() {
        let json = r#"{"$and":[]}"#;

        let query = parse_from_json(json).unwrap();

        let expected = Operator::And(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_empty_or_parse() {
        let json = r#"{"$or":[]}"#;

        let query = parse_from_json(json).unwrap();

        let expected = Operator::Or(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_empty_not_parse() {
        let json = r#"{"$not":{}}"#;

        let query = parse_from_json(json).unwrap();

        let expected = Operator::Not(Box::new(Operator::And(vec![])));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"{}":"{}"}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Eq(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_neq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"{}":{{"$neq":"{}"}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Neq(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_gt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"{}":{{"$gt":"{}"}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Gt(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_gte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"{}":{{"$gte":"{}"}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Gte(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_lt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"{}":{{"$lt":"{}"}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Lt(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_lte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"{}":{{"$lte":"{}"}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Lte(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_like_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"{}":{{"$like":"{}"}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Like(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_in_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"{}":{{"$in":["{}"]}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::In(name1, vec![value1]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_in_multiple_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let value2 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"{}":{{"$in":["{}","{}","{}"]}}}}"#, name1, value1, value2, value3);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::In(name1, vec![value1, value2, value3]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":"{}"}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Eq(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_neq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$neq":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Neq(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_gt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$gt":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Gt(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_gte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$gte":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Gte(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_lt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$lt":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Lt(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_lte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$lte":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Lte(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_like_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$like":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Like(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_in_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$in":["{}"]}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::In(name1, vec![value1])
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_not_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$and":[{{"$not":{{"{}":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Not(
                    Box::new(
                        Operator::Eq(name1, value1)
                    )
                )
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_short_and_with_multiple_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"{}":"{}","{}":"{}","{}":"{}"}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Eq(name1, value1),
                Operator::Eq(name2, value2),
                Operator::Eq(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":"{}"}},{{"{}":"{}"}},{{"{}":"{}"}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Eq(name1, value1),
                Operator::Eq(name2, value2),
                Operator::Eq(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_neq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Neq(name1, value1),
                Operator::Neq(name2, value2),
                Operator::Neq(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_gt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Gt(name1, value1),
                Operator::Gt(name2, value2),
                Operator::Gt(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_gte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Gte(name1, value1),
                Operator::Gte(name2, value2),
                Operator::Gte(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_lt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Lt(name1, value1),
                Operator::Lt(name2, value2),
                Operator::Lt(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_lte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Lte(name1, value1),
                Operator::Lte(name2, value2),
                Operator::Lte(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_like_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Like(name1, value1),
                Operator::Like(name2, value2),
                Operator::Like(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_in_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::In(name1, vec![value1]),
                Operator::In(name2, vec![value2]),
                Operator::In(name3, vec![value3])
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_not_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$and":[{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Not(
                    Box::new(
                        Operator::Eq(name1, value1)
                    )
                ),
                Operator::Not(
                    Box::new(
                        Operator::Eq(name2, value2)
                    )
                ),
                Operator::Not(
                    Box::new(
                        Operator::Eq(name3, value3)
                    )
                ),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_mixed_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);
        let name4 = random_string(10);
        let value4 = random_string(10);
        let name5 = random_string(10);
        let value5 = random_string(10);
        let name6 = random_string(10);
        let value6 = random_string(10);
        let name7 = random_string(10);
        let value7 = random_string(10);
        let name8 = random_string(10);
        let value8a = random_string(10);
        let value8b = random_string(10);
        let name9 = random_string(10);
        let value9 = random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":"{}"}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$in":["{}","{}"]}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
                           name4, value4,
                           name5, value5,
                           name6, value6,
                           name7, value7,
                           name8, value8a, value8b,
                           name9, value9,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::And(
            vec![
                Operator::Eq(name1, value1),
                Operator::Neq(name2, value2),
                Operator::Gt(name3, value3),
                Operator::Gte(name4, value4),
                Operator::Lt(name5, value5),
                Operator::Lte(name6, value6),
                Operator::Like(name7, value7),
                Operator::In(name8, vec![value8a, value8b]),
                Operator::Not(
                    Box::new(
                        Operator::Eq(name9, value9)
                    )
                ),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":"{}"}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Eq(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_neq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$neq":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Neq(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_gt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$gt":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Gt(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_gte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$gte":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Gte(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_lt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$lt":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Lt(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_lte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$lte":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Lte(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_like_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$like":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Like(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_in_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$in":["{}"]}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::In(name1, vec![value1])
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_not_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$or":[{{"$not":{{"{}":"{}"}}}}]}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Not(
                    Box::new(
                        Operator::Eq(name1, value1)
                    )
                )
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":"{}"}},{{"{}":"{}"}},{{"{}":"{}"}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Eq(name1, value1),
                Operator::Eq(name2, value2),
                Operator::Eq(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_neq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Neq(name1, value1),
                Operator::Neq(name2, value2),
                Operator::Neq(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_gt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Gt(name1, value1),
                Operator::Gt(name2, value2),
                Operator::Gt(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_gte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Gte(name1, value1),
                Operator::Gte(name2, value2),
                Operator::Gte(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_lt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Lt(name1, value1),
                Operator::Lt(name2, value2),
                Operator::Lt(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_lte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Lte(name1, value1),
                Operator::Lte(name2, value2),
                Operator::Lte(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_like_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Like(name1, value1),
                Operator::Like(name2, value2),
                Operator::Like(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_in_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::In(name1, vec![value1]),
                Operator::In(name2, vec![value2]),
                Operator::In(name3, vec![value3]),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_not_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);

        let json = format!(r#"{{"$or":[{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Not(
                    Box::new(
                        Operator::Eq(name1, value1)
                    )
                ),
                Operator::Not(
                    Box::new(
                        Operator::Eq(name2, value2)
                    )
                ),
                Operator::Not(
                    Box::new(
                        Operator::Eq(name3, value3)
                    )
                ),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_mixed_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);
        let name4 = random_string(10);
        let value4 = random_string(10);
        let name5 = random_string(10);
        let value5 = random_string(10);
        let name6 = random_string(10);
        let value6 = random_string(10);
        let name7 = random_string(10);
        let value7 = random_string(10);
        let name8 = random_string(10);
        let value8a = random_string(10);
        let value8b = random_string(10);
        let name9 = random_string(10);
        let value9 = random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":"{}"}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$in":["{}","{}"]}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
                           name4, value4,
                           name5, value5,
                           name6, value6,
                           name7, value7,
                           name8, value8a, value8b,
                           name9, value9,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Or(
            vec![
                Operator::Eq(name1, value1),
                Operator::Neq(name2, value2),
                Operator::Gt(name3, value3),
                Operator::Gte(name4, value4),
                Operator::Lt(name5, value5),
                Operator::Lte(name6, value6),
                Operator::Like(name7, value7),
                Operator::In(name8, vec![value8a, value8b]),
                Operator::Not(
                    Box::new(
                        Operator::Eq(name9, value9)
                    )
                ),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_eq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$not":{{"{}":"{}"}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Not(
            Box::new(
                Operator::Eq(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_neq_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$neq":"{}"}}}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Not(
            Box::new(
                Operator::Neq(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_gt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$gt":"{}"}}}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Not(
            Box::new(
                Operator::Gt(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_gte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$gte":"{}"}}}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Not(
            Box::new(
                Operator::Gte(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_lt_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$lt":"{}"}}}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Not(
            Box::new(
                Operator::Lt(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_lte_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$lte":"{}"}}}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Not(
            Box::new(
                Operator::Lte(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_like_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$like":"{}"}}}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Not(
            Box::new(
                Operator::Like(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_in_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$in":["{}"]}}}}}}"#, name1, value1);

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Not(
            Box::new(
                Operator::In(name1, vec![value1])
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_or_not_complex_case_parse() {
        let name1 = random_string(10);
        let value1 = random_string(10);
        let name2 = random_string(10);
        let value2 = random_string(10);
        let name3 = random_string(10);
        let value3 = random_string(10);
        let name4 = random_string(10);
        let value4 = random_string(10);
        let name5 = random_string(10);
        let value5 = random_string(10);
        let name6 = random_string(10);
        let value6 = random_string(10);
        let name7 = random_string(10);
        let value7 = random_string(10);
        let name8 = random_string(10);
        let value8 = random_string(10);

        let json = format!(r#"{{"$not":{{"$and":[{{"{}":"{}"}},{{"$or":[{{"{}":{{"$gt":"{}"}}}},{{"$not":{{"{}":{{"$lte":"{}"}}}}}},{{"$and":[{{"{}":{{"$lt":"{}"}}}},{{"$not":{{"{}":{{"$gte":"{}"}}}}}}]}}]}},{{"$not":{{"{}":{{"$like":"{}"}}}}}},{{"$and":[{{"{}":"{}"}},{{"$not":{{"{}":{{"$neq":"{}"}}}}}}]}}]}}}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
                           name4, value4,
                           name5, value5,
                           name6, value6,
                           name7, value7,
                           name8, value8,
        );

        let query = parse_from_json(&json).unwrap();

        let expected = Operator::Not(
            Box::new(
                Operator::And(
                    vec![
                        Operator::Eq(name1, value1),
                        Operator::Or(
                            vec![
                                Operator::Gt(name2, value2),
                                Operator::Not(
                                    Box::new(
                                        Operator::Lte(name3, value3)
                                    )
                                ),
                                Operator::And(
                                    vec![
                                        Operator::Lt(name4, value4),
                                        Operator::Not(
                                            Box::new(
                                                Operator::Gte(name5, value5)
                                            )
                                        ),
                                    ]
                                )
                            ]
                        ),
                        Operator::Not(
                            Box::new(
                                Operator::Like(name6, value6)
                            )
                        ),
                        Operator::And(
                            vec![
                                Operator::Eq(name7, value7),
                                Operator::Not(
                                    Box::new(
                                        Operator::Neq(name8, value8)
                                    )
                                ),
                            ]
                        )
                    ]
                )
            )
        );

        assert_eq!(query, expected);
    }
}