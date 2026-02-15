use crate::error::{Result, TaskVimError};

#[derive(Debug, PartialEq)]
pub enum Operator {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    Contains,
}

impl Operator {
    pub fn to_sql(&self) -> &'static str {
        match self {
            Operator::Eq => "=",
            Operator::Neq => "!=",
            Operator::Gt => ">",
            Operator::Lt => "<",
            Operator::Gte => ">=",
            Operator::Lte => "<=",
            Operator::Contains => "LIKE",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "=" => Some(Operator::Eq),
            "!=" => Some(Operator::Neq),
            ">" => Some(Operator::Gt),
            "<" => Some(Operator::Lt),
            ">=" => Some(Operator::Gte),
            "<=" => Some(Operator::Lte),
            "contains" => Some(Operator::Contains),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Filter {
    pub field: String,
    pub operator: Operator,
    pub value: String,
}

impl Filter {
    pub fn parse(input: &str) -> Result<Vec<Self>> {
        let mut filters = Vec::new();
        // Split by space, but we might need a better tokenizer later for quoted values
        // For now, assume simple space-separated "field=value" or "field>=value"
        // This is a naive parser.
        
        for part in input.split_whitespace() {
            let (op_str, _op_len, op_enum) = if part.contains(">=") {
                (">=", 2, Operator::Gte)
            } else if part.contains("<=") {
                ("<=", 2, Operator::Lte)
            } else if part.contains("!=") {
                ("!=", 2, Operator::Neq)
            } else if part.contains("=") {
                ("=", 1, Operator::Eq)
            } else if part.contains(">") {
                (">", 1, Operator::Gt)
            } else if part.contains("<") {
                ("<", 1, Operator::Lt)
            } else if part.contains("contains") {
                ("contains", 8, Operator::Contains)
            } else {
                continue; // Skip invalid parts
            };

            let parts: Vec<&str> = part.splitn(2, op_str).collect();
            if parts.len() != 2 {
                continue;
            }

            let field = parts[0].to_string();
            let value = parts[1].to_string();

            filters.push(Filter {
                field,
                operator: op_enum,
                value,
            });
        }

        Ok(filters)
    }

    pub fn to_sql_condition(&self) -> Result<(String, String)> {
        let col = match self.field.as_str() {
            "status" => "status",
            "priority" => "priority",
            "project" => "project",
            "due" => "due_date",
            "created" => "created_at",
            "tag" => return Err(TaskVimError::Validation("Tag filtering not yet implemented in SQL gen".into())),
             _ => return Err(TaskVimError::Validation(format!("Unknown field: {}", self.field))),
        };

        let val = if self.operator == Operator::Contains {
            format!("%{}%", self.value)
        } else {
            self.value.clone()
        };

        Ok((format!("{} {} ?", col, self.operator.to_sql()), val))
    }
}
