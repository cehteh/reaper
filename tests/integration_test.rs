use assert_cmd;
use std::collections::VecDeque;

macro_rules! object_vec {
    ( $($obj:expr),* ) => {
        {
            let mut v: Vec<Object> = vec![];
            $(
                v.push($obj.into());
            )*
            v
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Object {
    Number(f64),
    Bool(bool),
    Null,
    BytecodePtr(isize),
}

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Object::Bool(value)
    }
}

impl From<f64> for Object {
    fn from(value: f64) -> Self {
        Object::Number(value)
    }
}

fn fetch_output(path: &str) -> (VecDeque<String>, VecDeque<String>) {
    let mut spam = assert_cmd::Command::cargo_bin("reaper").unwrap();
    let assert = spam.arg(path).assert();
    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let split: VecDeque<String> = stdout
        .split('\n')
        .filter_map(|l| {
            if !l.is_empty() {
                Some(l.to_string())
            } else {
                None
            }
        })
        .collect();
    let filtered: VecDeque<String> = split
        .iter()
        .filter_map(|l| {
            if l.starts_with("dbg:") {
                Some(l.to_owned())
            } else {
                None
            }
        })
        .collect();
    (split, filtered)
}

#[test]
fn test_code_fragments() {
    let pairs = [
        (
            "./tests/cases/assignment01.reap",
            object_vec![2.0, 3.0, 4.0, 5.0, 20.0, Object::Null],
        ),
        ("./tests/cases/assignment02.reap", object_vec![6.0, 3.0]),
        (
            "./tests/cases/assignment03.reap",
            object_vec![69.0, 3.0, 12.0, 2.0, 1.0],
        ),
        ("./tests/cases/assignment04.reap", object_vec![10.0]),
        ("./tests/cases/fib20.reap", object_vec![6765.0]),
    ];
    for (path, output) in pairs {
        let (stdout, mut filtered) = fetch_output(path);
        let expected = output;
        for e in expected {
            assert!(filtered.pop_front().unwrap() == format!("dbg: {:?}", e));
        }
        assert!(stdout.back().unwrap() == "stack: []");
    }
}
