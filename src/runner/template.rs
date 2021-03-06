/*
 * hurl (https://hurl.dev)
 * Copyright (C) 2020 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use std::collections::HashMap;

use crate::core::ast::*;
use crate::core::common::Value;

use super::core::{Error, RunnerError};

impl Template {
    pub fn eval(self, variables: &HashMap<String, Value>) -> Result<String, Error> {
        let Template { elements, .. } = self;
        {
            let mut value = String::from("");
            for elem in elements {
                match elem.eval(variables) {
                    Ok(v) => value.push_str(v.as_str()),
                    Err(e) => return Err(e),
                }
            }
            Ok(value)
        }
    }
}

impl TemplateElement {
    pub fn eval(self, variables: &HashMap<String, Value>) -> Result<String, Error> {
        match self {
            TemplateElement::String { value, .. } => { Ok(value) }
            TemplateElement::Expression(Expr { variable: Variable { name, source_info }, .. }) => {
                match variables.get(&name as &str) {
                    Some(value) => if value.is_renderable() {
                        Ok(value.clone().to_string())
                    } else {
                        Err(Error { source_info, inner: RunnerError::UnrenderableVariable { value: value.to_string() }, assert: false })
                    },
                    _ => Err(Error { source_info, inner: RunnerError::TemplateVariableNotDefined { name }, assert: false }),
                }
            }
        }
    }
}

impl Value {
    pub fn is_renderable(&self) -> bool {
        match self {
            Value::Integer(_) | Value::Bool(_) | Value::Float(_, _) | Value::String(_) => true,
            _ => false,
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::core::common::SourceInfo;

    use super::*;

    fn template_element_expression() -> TemplateElement {
        // {{name}}
        TemplateElement::Expression(Expr {
            space0: Whitespace { value: "".to_string(), source_info: SourceInfo::init(1, 3, 1, 3) },
            variable: Variable { name: "name".to_string(), source_info: SourceInfo::init(1, 3, 1, 7) },
            space1: Whitespace { value: "".to_string(), source_info: SourceInfo::init(1, 7, 1, 7) },
        })
    }


    #[test]
    fn test_template_element() {
        let variables = HashMap::new();
        assert_eq!(TemplateElement::String { value: "World".to_string(), encoded: "World".to_string() }.eval(&variables).unwrap(),
                   "World".to_string()
        );

        let mut variables = HashMap::new();
        variables.insert("name".to_string(), Value::String("World".to_string()));
        assert_eq!(template_element_expression().eval(&variables).unwrap(), "World".to_string());
    }


    #[test]
    fn test_template_element_error() {
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), Value::List(vec![Value::Integer(1), Value::Integer(2)]));
        let error = template_element_expression().eval(&variables).err().unwrap();
        assert_eq!(error.source_info, SourceInfo::init(1, 3, 1, 7));
        assert_eq!(error.inner, RunnerError::UnrenderableVariable { value: "[1,2]".to_string() });
    }
}
