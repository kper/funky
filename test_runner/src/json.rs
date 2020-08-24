use funky::value::Value;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct TestFile {
    pub source_filename: String,
    commands: Vec<Command>,
}

impl TestFile {
    pub fn get_len_cases(&self) -> usize {
        self.commands
            .iter()
            .filter_map(|x| match x {
                Command::AssertReturn(w) => Some(w),
                _ => None,
            })
            .count()
    }

    pub fn get_cases(&self) -> impl Iterator<Item = &Command> {
        self.commands.iter().filter(|x| match x {
            Command::Module(m) => true,
            Command::AssertReturn(w) => true,
            _ => false,
        })
    }

    pub fn get_fs_names(&self) -> Vec<&String> {
        self
            .commands
            .iter()
            .filter_map(|x| match x {
                Command::Module(w) => Some(&w.filename),
                _ => None,
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub(crate) enum Command {
    #[serde(rename = "module")]
    Module(Module),
    #[serde(rename = "assert_return")]
    AssertReturn(AssertReturn),
    #[serde(rename = "assert_invalid")]
    AssertInvalid(AssertInvalid),
    #[serde(rename = "assert_trap")]
    AssertTrap, //TODO
    #[serde(rename = "assert_malformed")]
    AssertMalformed, //TODO
    #[serde(rename = "register")]
    Register, //TODO
    #[serde(rename = "assert_unlinkable")]
    AssertUnlinkable, //TODO
    #[serde(rename = "assert_exhaustion")]
    AssertExhaustion, //TODO
    #[serde(rename = "action")]
    Action, //TODO
    #[serde(rename = "assert_uninstantiable")]
    AssertUninstantiable, //TODO
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct AssertInvalid {
    line: usize,
    module_type: String,
    text: String,
    filename: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct Module {
    line: usize,
    pub filename: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct AssertReturn {
    pub line: usize,
    pub action: Action,
    expected: Vec<Argument>,
}

impl AssertReturn {
    pub fn get_args(&self) -> Vec<Value> {
        self.action
            .args
            .iter()
            .map(|w| w.clone().into())
            .collect::<Vec<_>>()
    }

    pub fn get_expected(&self) -> Vec<Value> {
        self.expected
            .iter()
            .map(|w| w.clone().into())
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct Action {
    pub field: String,
    #[serde(default = "Vec::new")]
    pub args: Vec<Argument>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct Argument {
    #[serde(rename = "type")]
    ty: String,
    value: String,
}

fn from_bits_f64(s: u64) -> f64 {
    f64::from_bits(s)
}

fn from_bits_f32(s: u32) -> f32 {
    f32::from_bits(s)
}

impl From<Argument> for Value {
    fn from(e: Argument) -> Value {
        use log::debug;

        let w = match e.ty.as_str() {
            "i32" => Value::I32((e.value.parse::<u32>().unwrap()) as i32),
            "i64" => Value::I64((e.value.parse::<u64>().unwrap()) as i64),
            "f32" => {
                if e.value.starts_with("nan") {
                    return Value::F32(f32::NAN);
                }

                if e.value.as_str() == "4286578688" {
                    return Value::F32(f32::NEG_INFINITY);
                }

                if e.value.as_str() == "2139095040" {
                    return Value::F32(f32::INFINITY);
                }

                Value::F32(from_bits_f32(e.value.parse().unwrap()))
            }

            "f64" => {
                if e.value.starts_with("nan") {
                    return Value::F64(f64::NAN);
                }

                if e.value.as_str() == "18442240474082181120" {
                    return Value::F64(f64::NEG_INFINITY);
                }

                if e.value.as_str() == "9218868437227405312" {
                    return Value::F64(f64::INFINITY);
                }

                Value::F64(from_bits_f64(e.value.parse().unwrap()))
            }
            _ => panic!(""),
        };

        debug!("Parsing Value {} {:?} = {:?}", e.ty, e.value, w);

        w
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /*
    #[test]
    fn parse_file() {
        let data = r#"
        {
 "source_filename": "f64.wast",
 "commands": [
  {"type": "module", "line": 5, "filename": "f64.0.wasm"}, 
  {"type": "assert_return", "line": 19, "action": {"type": "invoke", "field": "add", "args": [{"type": "f64", "value": "9223372036854775808"}, {"type": "f64", "value": "9223372036854775808"}]}, "expected": [{"type": "f64", "value": "9223372036854775808"}]}, 
  
        }"#;

        let v: TestFile = ::serde_json::from_str(data).unwrap();

        /*
        let acc = Action {
            field: "add".to_string(),
            args: vec![
                Argument {
                    ty: "f64".to_string(),
                    value: "9223372036854775808".to_string(),
                },
                Argument {
                    ty: "f64".to_string(),
                    value: "9223372036854775808".to_string(),
                },
            ],

        };

        let compare = TestFile {
            source_filename: "f64.wast".to_string(),
            commands: vec![
                Command::Module(Module {
                    line: 5,
                    filename: "f64.0.wasm".to_string(),
                }),
                Command::AssertReturn(AssertReturn {
                    line: 20,
                    action: acc,
            expected: vec![Argument {
                ty: "f64".to_string(),
                value: "9223372036854775808".to_string(),
            }],
                }),
            ],
        };

        assert_eq!(v, compare);
        */
    }
    */

    #[test]
    fn parse_action() {
        let data = r#"
        {
            "type": "invoke", "field": "add", "args": [{"type": "f64", "value": "9223372036854775808"}, {"type": "f64", "value": "9223372036854775808"}], "expected": [{"type": "f64", "value": "9223372036854775808"}]
        }"#;

        let v: Action = serde_json::from_str(data).unwrap();

        let compare = Action {
            field: "add".to_string(),
            args: vec![
                Argument {
                    ty: "f64".to_string(),
                    value: "9223372036854775808".to_string(),
                },
                Argument {
                    ty: "f64".to_string(),
                    value: "9223372036854775808".to_string(),
                },
            ],
        };

        assert_eq!(v, compare);
    }
}
