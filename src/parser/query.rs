use crate::core::ast::*;
use crate::core::common::Pos;
use crate::core::common::SourceInfo;

use super::combinators::*;
use super::cookiepath::cookiepath;
use super::ParseResult;
use super::primitives::*;
use super::reader::Reader;
use super::string::*;

pub fn query(reader: &mut Reader) -> ParseResult<'static, Query> {
    let start = reader.state.pos.clone();
    let value = query_value(reader)?;
    let end = reader.state.pos.clone();
    Ok(Query {
        source_info: SourceInfo { start, end },
        value,
    })
}


pub fn subquery(reader: &mut Reader) -> ParseResult<'static, Subquery> {
    let start = reader.state.pos.clone();
    let value = subquery_value(reader)?;
    let end = reader.state.pos.clone();
    Ok(Subquery {
        source_info: SourceInfo { start, end },
        value,
    })
}


fn query_value(reader: &mut Reader) -> ParseResult<'static, QueryValue> {
    choice(
        vec![
            status_query,
            header_query,
            cookie_query,
            body_query,
            xpath_query,
            jsonpath_query,
            regex_query,
            variable_query,
        ],
        reader,
    )
}


fn status_query(reader: &mut Reader) -> ParseResult<'static, QueryValue> {
    try_literal("status", reader)?;
    Ok(QueryValue::Status {})
}


fn header_query(reader: &mut Reader) -> ParseResult<'static, QueryValue> {
    try_literal("header", reader)?;
    let space0 = one_or_more_spaces(reader)?;
    let name = quoted_template(reader)?;
    Ok(QueryValue::Header { space0, name })
}


fn cookie_query(reader: &mut Reader) -> ParseResult<'static, QueryValue> {
    try_literal("cookie", reader)?;
    let space0 = one_or_more_spaces(reader)?;
    let start = reader.state.pos.clone();
    let s = quoted_string(reader)?;
    // todo should work with an encodedString in order to support escape sequence
    // or decode escape sequence with the cookiepath parser

    let mut cookiepath_reader = Reader::init(s.as_str());
    cookiepath_reader.state.pos = Pos { line: start.line, column: start.column + 1 };
    let expr = cookiepath(&mut cookiepath_reader)?;

    Ok(QueryValue::Cookie { space0, expr })
}


fn body_query(reader: &mut Reader) -> ParseResult<'static, QueryValue> {
    try_literal("body", reader)?;
    Ok(QueryValue::Body {})
}


fn xpath_query(reader: &mut Reader) -> ParseResult<'static, QueryValue> {
    try_literal("xpath", reader)?;
    let space0 = one_or_more_spaces(reader)?;
    let expr = quoted_template(reader)?;
    Ok(QueryValue::Xpath { space0, expr })
}


fn jsonpath_query(reader: &mut Reader) -> ParseResult<'static, QueryValue> {
    try_literal("jsonpath", reader)?;
    let space0 = one_or_more_spaces(reader)?;
    //let expr = jsonpath_expr(reader)?;
    //  let start = reader.state.pos.clone();
    let expr = quoted_template(reader)?;
//    let end = reader.state.pos.clone();
//    let expr = Template {
//        elements: template.elements.iter().map(|e| match e {
//            TemplateElement::String { value, encoded } => HurlTemplateElement::Literal {
//                value: HurlString2 { value: value.clone(), encoded: Some(encoded.clone()) }
//            },
//            TemplateElement::Expression(value) => HurlTemplateElement::Expression { value: value.clone() }
//        }).collect(),
//        quotes: true,
//        source_info: SourceInfo { start, end },
//    };

    Ok(QueryValue::Jsonpath { space0, expr })
}


fn regex_query(reader: &mut Reader) -> ParseResult<'static, QueryValue> {
    try_literal("regex", reader)?;
    let space0 = one_or_more_spaces(reader)?;
    let expr = quoted_template(reader)?;
    Ok(QueryValue::Regex { space0, expr })
}


fn variable_query(reader: &mut Reader) -> ParseResult<'static, QueryValue> {
    try_literal("variable", reader)?;
    let space0 = one_or_more_spaces(reader)?;
    let name = quoted_template(reader)?;
    Ok(QueryValue::Variable { space0, name })
}


fn subquery_value(reader: &mut Reader) -> ParseResult<'static, SubqueryValue> {
    choice(
        vec![
            regex_subquery,
        ],
        reader,
    )
}


fn regex_subquery(reader: &mut Reader) -> ParseResult<'static, SubqueryValue> {
    try_literal("regex", reader)?;
    let space0 = one_or_more_spaces(reader)?;
    let expr = quoted_template(reader)?;
    Ok(SubqueryValue::Regex { space0, expr })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let mut reader = Reader::init("status");
        assert_eq!(query(&mut reader).unwrap(), Query {
            source_info: SourceInfo::init(1, 1, 1, 7),
            value: QueryValue::Status {},
        });
    }

    #[test]
    fn test_status_query() {
        let mut reader = Reader::init("status");
        assert_eq!(query(&mut reader).unwrap(), Query {
            source_info: SourceInfo::init(1, 1, 1, 7),
            value: QueryValue::Status {},
        });
    }

    #[test]
    fn test_header_query() {
        let mut reader = Reader::init("header \"Foo\"");
        assert_eq!(
            header_query(&mut reader).unwrap(),
            QueryValue::Header {
                space0: Whitespace {
                    value: String::from(" "),
                    source_info: SourceInfo::init(1, 7, 1, 8),
                },
                name: Template {
                    quotes: true,
                    elements: vec![
                        TemplateElement::String {
                            value: "Foo".to_string(),
                            encoded: "Foo".to_string(),
                        }
                    ],
                    source_info: SourceInfo::init(1, 8, 1, 13),
                },
            }
        );
    }


    #[test]
    fn test_cookie_query() {
        let mut reader = Reader::init("cookie \"Foo[Domain]\"");
        assert_eq!(
            cookie_query(&mut reader).unwrap(),
            QueryValue::Cookie {
                space0: Whitespace {
                    value: String::from(" "),
                    source_info: SourceInfo::init(1, 7, 1, 8),
                },
                expr: CookiePath {
                    name: Template {
                        quotes: false,
                        elements: vec![
                            TemplateElement::String {
                                value: "Foo".to_string(),
                                encoded: "Foo".to_string(),
                            }
                        ],
                        source_info: SourceInfo::init(1, 9, 1, 12),
                    },
                    attribute: Some(CookieAttribute {
                        space0: Whitespace {
                            value: String::from(""),
                            source_info: SourceInfo::init(1, 13, 1, 13),
                        },
                        name: CookieAttributeName::Domain("Domain".to_string()),
                        space1: Whitespace {
                            value: String::from(""),
                            source_info: SourceInfo::init(1, 19, 1, 19),
                        },
                    }),

                },
            });
        assert_eq!(reader.state.cursor, 20);

        // todo test with escape sequence
        //let mut reader = Reader::init("cookie \"cookie\u{31}\"");
    }

    #[test]
    fn test_xpath_query() {
        let mut reader = Reader::init("xpath \"normalize-space(//head/title)\"");
        assert_eq!(
            xpath_query(&mut reader).unwrap(),
            QueryValue::Xpath {
                space0: Whitespace {
                    value: String::from(" "),
                    source_info: SourceInfo::init(1, 6, 1, 7),
                },
                expr: Template {
                    quotes: true,
                    elements: vec![
                        TemplateElement::String {
                            value: String::from("normalize-space(//head/title)"),
                            encoded: String::from("normalize-space(//head/title)"),
                        }
                    ],
                    source_info: SourceInfo::init(1, 7, 1, 38),
                },
            },
        );

        let mut reader = Reader::init("xpath \"normalize-space(//div[contains(concat(' ',normalize-space(@class),' '),' monthly-price ')])\"");
        assert_eq!(xpath_query(&mut reader).unwrap(), QueryValue::Xpath {
            space0: Whitespace { value: String::from(" "), source_info: SourceInfo::init(1, 6, 1, 7) },
            expr: Template {
                quotes: true,
                elements: vec![
                    TemplateElement::String {
                        value: String::from("normalize-space(//div[contains(concat(' ',normalize-space(@class),' '),' monthly-price ')])"),
                        encoded: String::from("normalize-space(//div[contains(concat(' ',normalize-space(@class),' '),' monthly-price ')])"),
                    }
                ],
                source_info: SourceInfo::init(1, 7, 1, 100),
            },

        });
    }

    #[test]
    fn test_jsonpath_query() {
        let mut reader = Reader::init("jsonpath \"$['statusCode']\"");
        assert_eq!(
            jsonpath_query(&mut reader).unwrap(),
            QueryValue::Jsonpath {
                space0: Whitespace {
                    value: String::from(" "),
                    source_info: SourceInfo::init(1, 9, 1, 10),
                },
                expr: Template {
                    elements: vec![
                        TemplateElement::String {
                            value: "$['statusCode']".to_string(),
                            encoded: "$['statusCode']".to_string(),
                        }
                    ],
                    quotes: true,
                    //delimiter: "\"".to_string(),
                    source_info: SourceInfo::init(1, 10, 1, 27),
                },
            },
        );
        let mut reader = Reader::init("jsonpath \"$.success\"");
        assert_eq!(
            jsonpath_query(&mut reader).unwrap(),
            QueryValue::Jsonpath {
                space0: Whitespace {
                    value: String::from(" "),
                    source_info: SourceInfo::init(1, 9, 1, 10),
                },
                expr: Template {
                    elements: vec![
                        TemplateElement::String {
                            value: "$.success".to_string(),
                            encoded: "$.success".to_string(),

                        }
                    ],
                    quotes: true,
                    //delimiter: "\"".to_string(),
                    source_info: SourceInfo::init(1, 10, 1, 21),
                },
            },
        );
    }
}