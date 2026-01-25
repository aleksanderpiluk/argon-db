mod schema {
    use crate::kv::{primary_key::PrimaryKeySchema, value_type::ValueTypeId};

    #[test]
    fn schema_too_much_columns() {
        let columns = (0..129).map(|_| ValueTypeId::I64).collect();

        let result = PrimaryKeySchema::new(columns);

        assert!(result.is_err());
    }

    #[test]
    fn schema_max_columns() {
        let columns = (0..128).map(|_| ValueTypeId::I64).collect();

        let result = PrimaryKeySchema::new(columns).expect("schema should be created");

        assert_eq!(result.column_count(), 128, "column count must be correct");
        result
            .get_column(127)
            .expect("last column should be returned");
        assert!(
            result.get_column(128).is_err(),
            "should fail on index out of bounds"
        );
    }
}

mod builder {
    use crate::kv::{
        primary_key::{PrimaryKeyBuilder, PrimaryKeySchema},
        value_type::ValueTypeId,
    };

    #[test]
    fn builder_test() {
        let schema = PrimaryKeySchema::new(vec![ValueTypeId::I32, ValueTypeId::Text])
            .expect("schema should be created");

        let mut builder = PrimaryKeyBuilder::new(&schema);
        todo!()
    }
}
