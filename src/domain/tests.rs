#[cfg(test)]
mod tests {
    use crate::domain::query::{Filter, Operator};

    #[test]
    fn test_filter_parsing() {
        let input = "status=todo priority>=3";
        let filters = Filter::parse(input).unwrap();
        
        assert_eq!(filters.len(), 2);
        
        assert_eq!(filters[0].field, "status");
        assert_eq!(filters[0].operator, Operator::Eq);
        assert_eq!(filters[0].value, "todo");

        assert_eq!(filters[1].field, "priority");
        assert_eq!(filters[1].operator, Operator::Gte);
        assert_eq!(filters[1].value, "3");
    }

    #[test]
    fn test_filter_sql_generation() {
        let f = Filter {
            field: "priority".to_string(),
            operator: Operator::Gt,
            value: "2".to_string(),
        };
        let (sql, val) = f.to_sql_condition().unwrap();
        assert_eq!(sql, "priority > ?");
        assert_eq!(val, "2");
    }
}
