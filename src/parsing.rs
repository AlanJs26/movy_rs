use pest::{Parser, Span};
use pest_derive::Parser;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{Error, Result};
use crate::movy::{Movy, Pipeline, Value};
use crate::pipes;

#[derive(Parser)]
#[grammar = "grammar/movy.pest"]
pub struct MovyParser;

#[derive(Debug)]
pub enum ValueToken<'a> {
    Number(f64),
    Boolean(bool),
    String(&'a str),
    Regex {
        content: &'a str,
        flags: Vec<&'a str>,
    },
}

#[derive(Debug)]
pub enum Token<'a> {
    Preamble(HashMap<&'a str, Token<'a>>),
    PipelineHeader {
        name: &'a str,
        properties: HashMap<&'a str, Token<'a>>,
    },
    PipelineRules(Vec<Token<'a>>),
    Pipeline {
        header: Box<Token<'a>>,
        rules: Box<Token<'a>>,
    },
    Filter {
        key: &'a str,
        value: Box<Token<'a>>,
        span: Span<'a>,
        raw: &'a str,
    },
    Action {
        key: &'a str,
        value: Box<Token<'a>>,
        span: Span<'a>,
        raw: &'a str,
    },
    Value(ValueToken<'a>),
}

#[derive(Debug)]
pub struct AST<'a> {
    pub preamble: Token<'a>,
    pub pipelines: Vec<Token<'a>>,
}

impl Pipeline {
    fn parse(token: &Token) -> Result<Pipeline> {
        let Token::Pipeline {
            header: header_token,
            rules: rules_token,
        } = token
        else {
            return Err(Error::Generic("Wrong pipeline token".into()));
        };

        let Token::PipelineHeader {
            name,
            properties: properties_token,
        } = header_token.as_ref()
        else {
            return Err(Error::Generic("Wrong Pipeline Header token".into()));
        };

        let Token::PipelineRules(rules_vec_token) = rules_token.as_ref() else {
            return Err(Error::Generic("Wrong Pipeline Rules token".into()));
        };

        let properties = properties_token
            .iter()
            .map(|(k, value_token)| {
                let key = String::from(*k);

                let Ok(value) = Value::parse(value_token) else {
                    return Err(Error::Generic("Wrong value token".into()));
                };

                Ok((key, value))
            })
            .collect::<Result<HashMap<String, Value>>>()?;

        let rules = rules_vec_token
            .iter()
            .map(|token| pipes::Pipe::parse(token))
            .collect::<Result<Vec<pipes::Pipe>>>()?;

        Ok(Pipeline::new(String::from(*name), properties, rules))
    }
}

impl Value {
    fn parse(token: &Token) -> Result<Self> {
        match token {
            Token::Value(ValueToken::Number(value)) => Ok(Value::Number(value.clone())),
            Token::Value(ValueToken::Boolean(value)) => Ok(Value::Boolean(value.clone())),
            Token::Value(ValueToken::String(value)) => Ok(Value::String((*value).into())),
            Token::Value(ValueToken::Regex { content, flags }) => Ok(Value::Regex {
                content: String::from(*content),
                flags: flags.iter().map(|value| String::from(*value)).collect(),
            }),
            _ => Err(Error::Generic("Wrong value token".into())),
        }
    }
}

impl pipes::Pipe {
    fn parse(token: &Token) -> Result<Self> {
        use pipes::{Basename, Move};

        match token {
            Token::Filter {
                key,
                value,
                span,
                raw,
            } => match *key {
                "basename" => Ok(Basename::new(
                    key,
                    Value::parse(value.as_ref())?,
                    (*raw).into(),
                )),
                _ => Err(Error::from_span("Unknown filter".to_string(), *span)),
            },
            Token::Action {
                key,
                value,
                span,
                raw,
            } => match *key {
                "move" => Ok(Move::new(key, Value::parse(value.as_ref())?, (*raw).into())),
                _ => Err(Error::from_span("Unknown action".to_string(), *span)),
            },
            _ => Err(Error::Generic(
                "Parsing rule with wrong type of token".into(),
            )),
        }
    }
}

impl Movy {
    pub fn from_file(filename: &str) -> Result<Movy> {
        let unparsed_file = fs::read_to_string(Path::new(filename))?;

        let movy_ast = MovyParser::parse_file(&unparsed_file)?;
        MovyParser::parse_ast(movy_ast)
    }
}

impl MovyParser {
    pub fn parse_file(filecontent: &str) -> Result<AST> {
        let movy = MovyParser::parse(Rule::file, filecontent)?.next().unwrap();

        use pest::iterators::Pair;

        fn parse_value(pair: Pair<Rule>) -> ValueToken {
            match pair.as_rule() {
                Rule::number => ValueToken::Number(pair.as_str().parse::<f64>().unwrap()),
                Rule::boolean => ValueToken::Boolean(match pair.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => unreachable!(),
                }),
                Rule::string => ValueToken::String(pair.as_str()),
                Rule::regex => {
                    let mut inner_pairs = pair.into_inner();

                    let content = inner_pairs.next().unwrap().as_str();
                    let flags = inner_pairs.map(|pair| pair.as_str()).collect();

                    ValueToken::Regex { content, flags }
                }
                _ => unreachable!(),
            }
        }

        fn parse_token(pair: Pair<Rule>) -> Token {
            match pair.as_rule() {
                Rule::preamble => Token::Preamble(
                    pair.into_inner()
                        .map(|pair| {
                            let mut inner_pairs = pair.into_inner();

                            let key = inner_pairs.next().unwrap().as_str();
                            let value = parse_token(inner_pairs.next().unwrap());
                            (key, value)
                        })
                        .collect(),
                ),
                Rule::pipeline => {
                    let mut inner_pairs = pair.into_inner();

                    let header = parse_token(inner_pairs.next().unwrap());
                    let rules =
                        Token::PipelineRules(inner_pairs.map(|token| parse_token(token)).collect());

                    Token::Pipeline {
                        header: Box::new(header),
                        rules: Box::new(rules),
                    }
                }
                Rule::pipeline_header => {
                    let mut inner_pairs = pair.into_inner();

                    let name = inner_pairs.next().unwrap().as_str();

                    let properties = inner_pairs
                        .map(|pair| {
                            let mut inner_properties = pair.into_inner();

                            let key = inner_properties.next().unwrap().as_str();
                            let value = parse_token(inner_properties.next().unwrap());

                            (key, value)
                        })
                        .collect();

                    Token::PipelineHeader { name, properties }
                }
                Rule::filter => {
                    let mut inner_pairs = pair.into_inner();

                    let raw = inner_pairs.as_str();
                    let key_pair = inner_pairs.next().unwrap();
                    let span = key_pair.as_span();

                    let key = key_pair.as_str();
                    let value = parse_token(inner_pairs.next().unwrap());

                    Token::Filter {
                        key,
                        value: Box::new(value),
                        span,
                        raw,
                    }
                }
                Rule::action => {
                    let mut inner_pairs = pair.into_inner();

                    let raw = inner_pairs.as_str();
                    let key_pair = inner_pairs.next().unwrap();
                    let span = key_pair.as_span();

                    let key = key_pair.as_str();
                    let value = parse_token(inner_pairs.next().unwrap());

                    Token::Action {
                        key,
                        value: Box::new(value),
                        span,
                        raw,
                    }
                }
                Rule::value => Token::Value(parse_value(pair.into_inner().next().unwrap())),
                rule => {
                    println!("{:?}", rule);
                    unreachable!()
                }
            }
        }

        let mut inner_pairs = movy.into_inner();

        Ok(AST {
            preamble: parse_token(inner_pairs.next().unwrap()),
            pipelines: inner_pairs
                .filter(|pair| pair.as_rule() != Rule::EOI)
                .map(parse_token)
                .collect(),
        })
    }

    pub fn parse_ast(ast: AST) -> Result<Movy> {
        let AST {
            preamble: preamble_token,
            pipelines: pipelines_token,
        } = ast;

        let preamble = MovyParser::parse_preamble(preamble_token)?;

        let pipelines = pipelines_token
            .iter()
            .map(Pipeline::parse)
            .collect::<Result<Vec<Pipeline>>>()?;

        Ok(Movy::new(preamble, pipelines))
    }

    fn parse_preamble(token: Token) -> Result<HashMap<String, Value>> {
        if let Token::Preamble(hashmap) = token {
            hashmap
                .iter()
                .map(|(k, v)| {
                    let key = String::from(*k);

                    let Ok(value) = Value::parse(v) else {
                        return Err(Error::Generic("Wrong value token".into()));
                    };

                    Ok((key, value))
                })
                .collect()
        } else {
            return Err(Error::Generic("Wrong preamble token".into()));
        }
    }
}
